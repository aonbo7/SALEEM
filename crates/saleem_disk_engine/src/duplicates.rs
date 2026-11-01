use crate::models::{
    DuplicateFileMember, DuplicateGroup, DuplicateRemovalGroupSelection, DuplicateRemovalReport,
    DuplicateScanRequest, DuplicateScanResult,
};
use crate::scanner::classify_extension_category;
use chrono::{DateTime, Utc};
use saleem_core::paths::{
    assert_safe_for_modification, is_protected_system_path, is_reparse_point,
    normalize_lexical_windows_path,
};
use saleem_core::{CancellationToken, SaleemError, SaleemResult};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::Instant;
use uuid::Uuid;

const SAMPLE_CHUNK_SIZE: usize = 8192; // 8 KB head + 8 KB tail
const STREAM_BUFFER_SIZE: usize = 65536; // 64 KB streaming buffer

#[derive(Debug, Clone)]
struct CandidateFile {
    path: PathBuf,
    path_str: String,
    file_name: String,
    parent_dir: String,
    extension: String,
    modified_at: String,
    _size: u64,
    volume_and_index: Option<(u32, u64)>,
    hardlink_count: u32,
}

/// Computes a fast partial SHA-256 digest of the first 8 KB and last 8 KB of a file.
pub fn compute_partial_hash(path: &Path, file_size: u64) -> SaleemResult<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    hasher.update(file_size.to_le_bytes());

    if file_size <= (SAMPLE_CHUNK_SIZE as u64 * 2) {
        let mut buf = Vec::with_capacity(file_size as usize);
        file.read_to_end(&mut buf)?;
        hasher.update(&buf);
    } else {
        let mut head = [0u8; SAMPLE_CHUNK_SIZE];
        file.read_exact(&mut head)?;
        hasher.update(head);

        file.seek(SeekFrom::End(-(SAMPLE_CHUNK_SIZE as i64)))?;
        let mut tail = [0u8; SAMPLE_CHUNK_SIZE];
        file.read_exact(&mut tail)?;
        hasher.update(tail);
    }

    Ok(hex::encode(hasher.finalize()))
}

/// Computes the full cryptographic SHA-256 hash of a file with cooperative cancellation support.
pub fn compute_full_sha256(path: &Path, cancel_token: &CancellationToken) -> SaleemResult<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; STREAM_BUFFER_SIZE];

    loop {
        if cancel_token.is_cancelled() {
            return Err(SaleemError::ScanCancelled);
        }
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(hex::encode(hasher.finalize()))
}

/// Queries Windows NTFS file index and hardlink count via `GetFileInformationByHandle`.
fn inspect_hardlink_info(path: &Path) -> (Option<(u32, u64)>, u32) {
    #[cfg(windows)]
    {
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::Foundation::HANDLE;
        use windows_sys::Win32::Storage::FileSystem::{
            GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION,
        };

        if let Ok(file) = File::open(path) {
            let handle = file.as_raw_handle() as HANDLE;
            let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
            let ok = unsafe { GetFileInformationByHandle(handle, &mut info) };
            if ok != 0 {
                let file_index = ((info.nFileIndexHigh as u64) << 32) | (info.nFileIndexLow as u64);
                return (
                    Some((info.dwVolumeSerialNumber, file_index)),
                    info.nNumberOfLinks.max(1),
                );
            }
        }
        (None, 1)
    }
    #[cfg(not(windows))]
    {
        let _ = path;
        (None, 1)
    }
}

/// Multi-stage duplicate file finder:
/// 1. Group by exact `logical_size` (ignoring 0-byte and unique sizes).
/// 2. Filter by fast 8 KB head+tail partial hash.
/// 3. Verify with full streaming SHA-256 hash and detect NTFS hardlinks.
pub fn find_duplicates_with_progress<F>(
    req: &DuplicateScanRequest,
    cancel_token: &CancellationToken,
    mut on_progress: F,
) -> SaleemResult<DuplicateScanResult>
where
    F: FnMut(u64, u64, u64, &str, u64),
{
    let normalized_root = normalize_lexical_windows_path(&req.target_path)?;
    let root_path = PathBuf::from(&normalized_root);

    if !root_path.exists() {
        return Err(SaleemError::PathNotFound(normalized_root));
    }

    let started_at = Utc::now().to_rfc3339();
    let timer = Instant::now();
    let scan_id = Uuid::new_v4().to_string();
    let min_size = req.min_file_size_bytes.unwrap_or(1024).max(1);
    let max_groups = req.max_groups.unwrap_or(150).clamp(10, 500);

    let mut size_buckets: HashMap<u64, Vec<CandidateFile>> = HashMap::new();
    let mut files_evaluated: u64 = 0;
    let mut folders_seen: u64 = 0;
    let mut bytes_seen: u64 = 0;
    let mut errors_count: u64 = 0;

    // Stage 1: Iterative directory walk & size grouping
    let mut stack = vec![root_path];
    let mut tick: u64 = 0;

    while let Some(curr_dir) = stack.pop() {
        if cancel_token.is_cancelled() {
            return Err(SaleemError::ScanCancelled);
        }

        let curr_str = curr_dir.to_string_lossy().into_owned();
        // Skip protected Windows system directories from duplicate cleanup candidates
        if is_protected_system_path(&curr_str) && stack.len() > 0 {
            continue;
        }

        let rd = match std::fs::read_dir(&curr_dir) {
            Ok(r) => r,
            Err(_) => {
                errors_count += 1;
                continue;
            }
        };

        for entry_res in rd {
            if cancel_token.is_cancelled() {
                return Err(SaleemError::ScanCancelled);
            }
            let entry = match entry_res {
                Ok(e) => e,
                Err(_) => {
                    errors_count += 1;
                    continue;
                }
            };
            let entry_path = entry.path();
            if is_reparse_point(&entry_path) {
                continue;
            }

            let meta = match entry.metadata() {
                Ok(m) => m,
                Err(_) => {
                    errors_count += 1;
                    continue;
                }
            };

            if meta.is_dir() {
                folders_seen += 1;
                stack.push(entry_path);
            } else if meta.is_file() {
                let sz = meta.len();
                if sz < min_size {
                    continue;
                }
                let path_str = entry_path.to_string_lossy().into_owned();
                if is_protected_system_path(&path_str) {
                    continue;
                }

                files_evaluated += 1;
                bytes_seen = bytes_seen.saturating_add(sz);

                let file_name = entry.file_name().to_string_lossy().into_owned();
                let extension = entry_path
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e.trim().to_ascii_lowercase())
                    .filter(|e| !e.is_empty())
                    .unwrap_or_else(|| "bin".to_string());
                let modified_at = meta
                    .modified()
                    .ok()
                    .map(|st| DateTime::<Utc>::from(st).to_rfc3339())
                    .unwrap_or_else(|| started_at.clone());

                size_buckets.entry(sz).or_default().push(CandidateFile {
                    path: entry_path,
                    path_str,
                    file_name,
                    parent_dir: curr_str.clone(),
                    extension,
                    modified_at,
                    _size: sz,
                    volume_and_index: None,
                    hardlink_count: 1,
                });
            }

            tick += 1;
            if tick >= 250 {
                tick = 0;
                on_progress(
                    files_evaluated,
                    folders_seen,
                    bytes_seen,
                    &curr_str,
                    errors_count,
                );
            }
        }
    }

    // Retain only size buckets with >= 2 files
    let candidate_buckets: Vec<(u64, Vec<CandidateFile>)> = size_buckets
        .into_iter()
        .filter(|(_, list)| list.len() >= 2)
        .collect();
    let candidate_size_groups = candidate_buckets.len() as u64;

    let mut partial_hashes_computed: u64 = 0;
    let mut full_hashes_computed: u64 = 0;
    let mut hardlinks_flagged: u64 = 0;
    let mut groups: Vec<DuplicateGroup> = Vec::new();

    // Stage 2 & Stage 3: Partial Hash -> Full SHA-256 Verification
    for (file_size, candidates) in candidate_buckets {
        if cancel_token.is_cancelled() {
            return Err(SaleemError::ScanCancelled);
        }

        // Stage 2: Partial Hash Grouping
        let mut partial_map: HashMap<String, Vec<CandidateFile>> = HashMap::new();
        for cand in candidates {
            if cancel_token.is_cancelled() {
                return Err(SaleemError::ScanCancelled);
            }
            partial_hashes_computed += 1;
            match compute_partial_hash(&cand.path, file_size) {
                Ok(ph) => {
                    partial_map.entry(ph).or_default().push(cand);
                }
                Err(_) => {
                    errors_count += 1;
                }
            }
        }

        // Stage 3: Full SHA-256 on Partial Matches
        for (_ph, partial_matches) in partial_map.into_iter().filter(|(_, v)| v.len() >= 2) {
            let mut sha_map: HashMap<String, Vec<CandidateFile>> = HashMap::new();
            for mut cand in partial_matches {
                if cancel_token.is_cancelled() {
                    return Err(SaleemError::ScanCancelled);
                }
                on_progress(
                    files_evaluated,
                    folders_seen,
                    bytes_seen,
                    &cand.path_str,
                    errors_count,
                );
                full_hashes_computed += 1;
                let (vol_idx, hl_count) = inspect_hardlink_info(&cand.path);
                cand.volume_and_index = vol_idx;
                cand.hardlink_count = hl_count;

                match compute_full_sha256(&cand.path, cancel_token) {
                    Ok(full_sha) => {
                        sha_map.entry(full_sha).or_default().push(cand);
                    }
                    Err(SaleemError::ScanCancelled) => return Err(SaleemError::ScanCancelled),
                    Err(_) => {
                        errors_count += 1;
                    }
                }
            }

            for (sha256, mut exact_matches) in sha_map.into_iter().filter(|(_, v)| v.len() >= 2) {
                // Sort copies deterministically by modified_at then path
                exact_matches.sort_by(|a, b| {
                    a.modified_at
                        .cmp(&b.modified_at)
                        .then_with(|| a.path_str.cmp(&b.path_str))
                });

                let mut seen_inodes: HashSet<(u32, u64)> = HashSet::new();
                let mut independent_copies_count: u64 = 0;
                let mut members: Vec<DuplicateFileMember> = Vec::with_capacity(exact_matches.len());

                for (idx, item) in exact_matches.iter().enumerate() {
                    let is_hardlink_alias = if let Some(inode) = item.volume_and_index {
                        if !seen_inodes.insert(inode) {
                            hardlinks_flagged += 1;
                            true
                        } else {
                            independent_copies_count += 1;
                            false
                        }
                    } else {
                        independent_copies_count += 1;
                        false
                    };

                    members.push(DuplicateFileMember {
                        path: item.path_str.clone(),
                        file_name: item.file_name.clone(),
                        parent_dir: item.parent_dir.clone(),
                        modified_at: item.modified_at.clone(),
                        is_hardlink_alias,
                        hardlink_count: item.hardlink_count,
                        suggested_keep: idx == 0,
                    });
                }

                let wasted_bytes = if independent_copies_count > 1 {
                    (independent_copies_count - 1).saturating_mul(file_size)
                } else {
                    0
                };

                let ext = exact_matches[0].extension.clone();
                let category = classify_extension_category(&ext).to_string();

                groups.push(DuplicateGroup {
                    group_id: Uuid::new_v4().to_string(),
                    sha256,
                    extension: ext,
                    category,
                    file_size_bytes: file_size,
                    wasted_bytes,
                    copies: members,
                });
            }
        }
    }

    groups.sort_by(|a, b| b.wasted_bytes.cmp(&a.wasted_bytes));
    groups.truncate(max_groups);

    let total_duplicate_groups = groups.len();
    let total_duplicate_files: usize = groups.iter().map(|g| g.copies.len()).sum();
    let total_reclaimable_bytes: u64 = groups.iter().map(|g| g.wasted_bytes).sum();
    let is_partial = errors_count > 0;
    let status = if is_partial {
        "PARTIAL_COMPLETED".to_string()
    } else {
        "COMPLETED".to_string()
    };

    Ok(DuplicateScanResult {
        scan_id,
        target_path: normalized_root,
        status,
        started_at,
        completed_at: Utc::now().to_rfc3339(),
        duration_ms: timer.elapsed().as_millis() as u64,
        files_evaluated,
        candidate_size_groups,
        partial_hashes_computed,
        full_hashes_computed,
        hardlinks_flagged,
        errors_count,
        is_partial,
        total_duplicate_groups,
        total_duplicate_files,
        total_reclaimable_bytes,
        groups,
    })
}

/// Safely re-verifies (TOCTOU guard) and removes user-selected duplicate copies while guaranteeing
/// that a valid, unmodified reference copy remains on disk for every group.
pub fn verify_and_remove_duplicate_selections(
    selections: &[DuplicateRemovalGroupSelection],
) -> SaleemResult<DuplicateRemovalReport> {
    let job_id = Uuid::new_v4().to_string();
    let token = CancellationToken::new();

    let mut succeeded_count = 0usize;
    let mut failed_count = 0usize;
    let mut bytes_reclaimed = 0u64;
    let mut deleted_paths = Vec::new();
    let mut failed_items = Vec::new();

    for group in selections {
        if group.delete_paths.is_empty() {
            continue;
        }

        let norm_ref = match normalize_lexical_windows_path(&group.retained_reference_path) {
            Ok(p) => p,
            Err(e) => {
                for dp in &group.delete_paths {
                    failed_count += 1;
                    failed_items.push((dp.clone(), e.to_string()));
                }
                continue;
            }
        };

        // Rule: Never allow deleting the retained reference copy itself!
        if group.delete_paths.iter().any(|p| {
            p.eq_ignore_ascii_case(&norm_ref)
                || p.eq_ignore_ascii_case(&group.retained_reference_path)
        }) {
            return Err(SaleemError::InvalidInput(format!(
                "Safety violation: Cannot delete retained reference copy '{}'",
                norm_ref
            )));
        }

        // Re-verify that the retained reference copy still exists and matches size + SHA-256
        let ref_path = PathBuf::from(&norm_ref);
        let ref_meta = match std::fs::metadata(&ref_path) {
            Ok(m) if m.is_file() => m,
            _ => {
                for dp in &group.delete_paths {
                    failed_count += 1;
                    failed_items.push((
                        dp.clone(),
                        format!("Retained copy '{}' no longer exists", norm_ref),
                    ));
                }
                continue;
            }
        };

        if ref_meta.len() != group.file_size_bytes {
            for dp in &group.delete_paths {
                failed_count += 1;
                failed_items.push((
                    dp.clone(),
                    "Retained reference file size changed since scan".to_string(),
                ));
            }
            continue;
        }

        let ref_sha = match compute_full_sha256(&ref_path, &token) {
            Ok(h) => h,
            Err(e) => {
                for dp in &group.delete_paths {
                    failed_count += 1;
                    failed_items.push((dp.clone(), e.to_string()));
                }
                continue;
            }
        };

        if !ref_sha.eq_ignore_ascii_case(&group.group_sha256) {
            for dp in &group.delete_paths {
                failed_count += 1;
                failed_items.push((
                    dp.clone(),
                    "Retained reference file SHA-256 changed since scan".to_string(),
                ));
            }
            continue;
        }

        // Now re-verify and remove each candidate copy in `delete_paths`
        for raw_del in &group.delete_paths {
            let safe_del_path = match assert_safe_for_modification(raw_del) {
                Ok(p) => p,
                Err(e) => {
                    failed_count += 1;
                    failed_items.push((raw_del.clone(), e.to_string()));
                    continue;
                }
            };

            let del_meta = match std::fs::metadata(&safe_del_path) {
                Ok(m) if m.is_file() => m,
                _ => {
                    failed_count += 1;
                    failed_items.push((raw_del.clone(), "File not found".to_string()));
                    continue;
                }
            };

            if del_meta.len() != group.file_size_bytes {
                failed_count += 1;
                failed_items.push((
                    raw_del.clone(),
                    SaleemError::FileChangedSincePreview(
                        "File size changed since duplicate scan".into(),
                    )
                    .to_string(),
                ));
                continue;
            }

            match compute_full_sha256(&safe_del_path, &token) {
                Ok(del_sha) if del_sha.eq_ignore_ascii_case(&group.group_sha256) => {
                    match std::fs::remove_file(&safe_del_path) {
                        Ok(()) => {
                            succeeded_count += 1;
                            bytes_reclaimed = bytes_reclaimed.saturating_add(group.file_size_bytes);
                            deleted_paths.push(safe_del_path.to_string_lossy().into_owned());
                        }
                        Err(e) => {
                            failed_count += 1;
                            failed_items.push((raw_del.clone(), e.to_string()));
                        }
                    }
                }
                Ok(_) => {
                    failed_count += 1;
                    failed_items.push((
                        raw_del.clone(),
                        SaleemError::FileChangedSincePreview(
                            "File content hash changed since duplicate scan".into(),
                        )
                        .to_string(),
                    ));
                }
                Err(e) => {
                    failed_count += 1;
                    failed_items.push((raw_del.clone(), e.to_string()));
                }
            }
        }
    }

    Ok(DuplicateRemovalReport {
        job_id,
        succeeded_count,
        failed_count,
        bytes_reclaimed,
        deleted_paths,
        failed_items,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_duplicate_finder_matches_by_content_not_name_and_ignores_same_size_diff_content() {
        let dir = tempfile::tempdir().unwrap();
        let sub1 = dir.path().join("FolderA");
        let sub2 = dir.path().join("FolderB");
        fs::create_dir_all(&sub1).unwrap();
        fs::create_dir_all(&sub2).unwrap();

        let payload_dup = vec![0x5Au8; 25_000];
        let payload_diff = vec![0xA5u8; 25_000];

        // Two identical files with completely different names
        fs::write(sub1.join("original_report.pdf"), &payload_dup).unwrap();
        fs::write(sub2.join("copy_with_other_name.pdf"), &payload_dup).unwrap();

        // Third file with the EXACT same size (25,000 bytes) and same name as original, but different content!
        fs::write(sub2.join("original_report.pdf"), &payload_diff).unwrap();

        let token = CancellationToken::new();
        let req = DuplicateScanRequest {
            target_path: dir.path().to_string_lossy().into_owned(),
            min_file_size_bytes: Some(1024),
            max_groups: Some(50),
        };

        let res = find_duplicates_with_progress(&req, &token, |_, _, _, _, _| {}).unwrap();
        assert_eq!(res.total_duplicate_groups, 1);
        assert_eq!(res.groups[0].copies.len(), 2);
        assert_eq!(res.groups[0].wasted_bytes, 25_000);
        // Exactly one copy must have suggested_keep = true
        assert_eq!(
            res.groups[0]
                .copies
                .iter()
                .filter(|c| c.suggested_keep)
                .count(),
            1
        );
    }

    #[test]
    fn test_duplicate_removal_enforces_retained_copy_and_toctou_verification() {
        let dir = tempfile::tempdir().unwrap();
        let keep_file = dir.path().join("keep.bin");
        let del_file = dir.path().join("delete_me.bin");
        let modified_file = dir.path().join("modified_after_scan.bin");

        let content = vec![b'X'; 4096];
        fs::write(&keep_file, &content).unwrap();
        fs::write(&del_file, &content).unwrap();
        fs::write(&modified_file, &content).unwrap();

        let token = CancellationToken::new();
        let sha = compute_full_sha256(&keep_file, &token).unwrap();

        // 1. Attempting to include the retained copy in `delete_paths` must fail immediately
        let bad_sel = DuplicateRemovalGroupSelection {
            group_sha256: sha.clone(),
            file_size_bytes: 4096,
            retained_reference_path: keep_file.to_string_lossy().into_owned(),
            delete_paths: vec![keep_file.to_string_lossy().into_owned()],
        };
        assert!(verify_and_remove_duplicate_selections(&[bad_sel]).is_err());
        assert!(keep_file.exists());

        // 2. Modify `modified_file` after scan so TOCTOU protection blocks its deletion
        fs::write(&modified_file, vec![b'Y'; 4096]).unwrap();

        let valid_sel = DuplicateRemovalGroupSelection {
            group_sha256: sha,
            file_size_bytes: 4096,
            retained_reference_path: keep_file.to_string_lossy().into_owned(),
            delete_paths: vec![
                del_file.to_string_lossy().into_owned(),
                modified_file.to_string_lossy().into_owned(),
            ],
        };

        let report = verify_and_remove_duplicate_selections(&[valid_sel]).unwrap();
        assert_eq!(report.succeeded_count, 1);
        assert_eq!(report.failed_count, 1);
        assert_eq!(report.bytes_reclaimed, 4096);
        assert!(keep_file.exists(), "Retained copy must still exist");
        assert!(!del_file.exists(), "Verified duplicate must be removed");
        assert!(
            modified_file.exists(),
            "Modified file must be protected by TOCTOU guard"
        );
    }
}
