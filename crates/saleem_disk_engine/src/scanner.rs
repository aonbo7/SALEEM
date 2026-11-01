use crate::models::{
    CategoryStat, DiskScanRequest, DiskScanResult, ExtensionStat, ScannedFileEntry,
    ScannedFolderSummary,
};
use chrono::{DateTime, Utc};
use saleem_core::paths::{is_reparse_point, normalize_lexical_windows_path};
use saleem_core::{CancellationToken, SaleemError, SaleemResult};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::path::{Path, PathBuf};
use std::time::Instant;
use uuid::Uuid;

/// Wrapper for keeping a bounded min-heap of the largest `N` files without storing millions of files in RAM.
#[derive(Debug, Clone)]
struct MinHeapFileItem(ScannedFileEntry);

impl PartialEq for MinHeapFileItem {
    fn eq(&self, other: &Self) -> bool {
        self.0.logical_size == other.0.logical_size && self.0.path == other.0.path
    }
}
impl Eq for MinHeapFileItem {}

impl PartialOrd for MinHeapFileItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MinHeapFileItem {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering by logical_size so BinaryHeap acts as a min-heap
        other
            .0
            .logical_size
            .cmp(&self.0.logical_size)
            .then_with(|| other.0.path.cmp(&self.0.path))
    }
}

#[derive(Default, Clone)]
struct FolderAccum {
    path: String,
    name: String,
    logical_size: u64,
    allocated_size: u64,
    files_count: u64,
    folders_count: u64,
    is_partial: bool,
}

#[derive(Default)]
struct ExtAccum {
    category: String,
    files_count: u64,
    logical_size: u64,
    allocated_size: u64,
}

/// Computes the physical allocated size of a file on Windows using `GetCompressedFileSizeW`,
/// falling back to 4096-byte cluster rounding if unavailable.
pub fn compute_allocated_size(path: &Path, logical_size: u64) -> u64 {
    if logical_size == 0 {
        return 0;
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::Storage::FileSystem::GetCompressedFileSizeW;

        const INVALID_FILE_SIZE: u32 = 0xFFFF_FFFF;
        let wide: Vec<u16> = path
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        let mut high: u32 = 0;
        let low = unsafe { GetCompressedFileSizeW(wide.as_ptr(), &mut high) };
        if low != INVALID_FILE_SIZE {
            let combined = ((high as u64) << 32) | (low as u64);
            if combined > 0 {
                return combined;
            }
        }
    }
    // Standard 4 KB NTFS cluster rounding fallback
    const CLUSTER_SIZE: u64 = 4096;
    ((logical_size + CLUSTER_SIZE - 1) / CLUSTER_SIZE) * CLUSTER_SIZE
}

/// Classifies a normalized lowercase file extension into a human-friendly storage category.
pub fn classify_extension_category(ext: &str) -> &'static str {
    match ext {
        "exe" | "dll" | "msi" | "msix" | "appx" | "sys" | "bat" | "cmd" | "ps1" | "com" | "scr" => {
            "Applications & Binaries"
        }
        "pak" | "vpk" | "cas" | "ucas" | "utoc" | "rpf" | "upk" | "assets" | "bundle" | "forge"
        | "wad" | "ba2" | "bsa" | "pck" => "Games & Engine Assets",
        "vhdx" | "vhd" | "vmdk" | "vdi" | "qcow2" | "iso" | "wim" | "esd" | "img" => {
            "Virtual Disks & OS Images"
        }
        "safetensors" | "gguf" | "onnx" | "pt" | "pth" | "ckpt" | "tflite" => "AI Models & Weights",
        "mp4" | "mkv" | "avi" | "mov" | "webm" | "wmv" | "flv" | "mp3" | "wav" | "flac" | "ogg"
        | "m4a" | "aac" | "m4v" => "Video & Audio Media",
        "png" | "jpg" | "jpeg" | "webp" | "gif" | "bmp" | "svg" | "ico" | "psd" | "raw"
        | "tiff" | "blend" | "fbx" => "Images & 3D Design",
        "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" | "cab" | "zst" => {
            "Archives & Compressed"
        }
        "sqlite" | "sqlite3" | "db" | "mdf" | "ldf" | "bak" | "parquet" => "Databases & Backups",
        "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "txt" | "md" | "rtf" | "csv"
        | "json" | "xml" | "yaml" | "yml" | "toml" => "Documents & Data",
        "rs" | "ts" | "tsx" | "js" | "jsx" | "py" | "c" | "cpp" | "h" | "hpp" | "cs" | "go"
        | "java" | "html" | "css" | "sql" | "pdb" | "rlib" => "Source Code & Build Artifacts",
        "log" | "tmp" | "temp" | "dmp" | "hdmp" | "mdmp" | "old" | "cache" | "etl" => {
            "Logs, Temp & Crash Dumps"
        }
        _ => "Other Files",
    }
}

/// Performs a bounded-memory, cancellable directory tree scan with reparse-point loop prevention
/// and partial-scan accuracy tracking.
pub fn scan_directory_with_progress<F>(
    req: &DiskScanRequest,
    cancel_token: &CancellationToken,
    mut on_progress: F,
) -> SaleemResult<DiskScanResult>
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

    let max_top_files = req.max_top_files.unwrap_or(100).clamp(10, 500);
    let max_top_folders = req.max_top_folders.unwrap_or(60).clamp(10, 300);
    let exclude_patterns: Vec<String> = req
        .exclude_patterns
        .clone()
        .unwrap_or_default()
        .into_iter()
        .map(|s| s.to_ascii_lowercase())
        .collect();

    let mut top_files_heap: BinaryHeap<MinHeapFileItem> =
        BinaryHeap::with_capacity(max_top_files + 1);
    let mut immediate_children_map: HashMap<String, FolderAccum> = HashMap::new();
    let mut second_level_folders_map: HashMap<String, FolderAccum> = HashMap::new();
    let mut root_files_accum = FolderAccum {
        path: normalized_root.clone(),
        name: "[Files in Root]".to_string(),
        ..Default::default()
    };
    let mut ext_map: HashMap<String, ExtAccum> = HashMap::new();

    let mut total_logical_bytes: u64 = 0;
    let mut total_allocated_bytes: u64 = 0;
    let mut files_count: u64 = 0;
    let mut folders_count: u64 = 0;
    let mut errors_count: u64 = 0;
    let mut reparse_points_skipped: u64 = 0;
    let mut sample_inaccessible_paths: Vec<String> = Vec::new();

    // Iterative stack-based traversal to avoid stack overflow on deeply nested trees
    // Each stack item: (directory_path, immediate_child_key, second_level_key)
    let mut stack: Vec<(PathBuf, Option<String>, Option<String>)> =
        vec![(root_path.clone(), None, None)];

    let mut items_since_last_progress: u64 = 0;

    while let Some((curr_dir, imm_key, sec_key)) = stack.pop() {
        if cancel_token.is_cancelled() {
            return Err(SaleemError::ScanCancelled);
        }

        let curr_dir_str = curr_dir.to_string_lossy().into_owned();
        if !exclude_patterns.is_empty() {
            let lower_dir = curr_dir_str.to_ascii_lowercase();
            if exclude_patterns.iter().any(|pat| lower_dir.contains(pat)) {
                continue;
            }
        }

        let read_dir = match std::fs::read_dir(&curr_dir) {
            Ok(rd) => rd,
            Err(_) => {
                errors_count += 1;
                if sample_inaccessible_paths.len() < 50 {
                    sample_inaccessible_paths.push(curr_dir_str.clone());
                }
                if let Some(ref k) = imm_key {
                    if let Some(acc) = immediate_children_map.get_mut(k) {
                        acc.is_partial = true;
                    }
                }
                if let Some(ref k2) = sec_key {
                    if let Some(acc) = second_level_folders_map.get_mut(k2) {
                        acc.is_partial = true;
                    }
                }
                continue;
            }
        };

        for entry_res in read_dir {
            if cancel_token.is_cancelled() {
                return Err(SaleemError::ScanCancelled);
            }

            let entry = match entry_res {
                Ok(e) => e,
                Err(_) => {
                    errors_count += 1;
                    if let Some(ref k) = imm_key {
                        if let Some(acc) = immediate_children_map.get_mut(k) {
                            acc.is_partial = true;
                        }
                    }
                    continue;
                }
            };

            let entry_path = entry.path();
            let entry_path_str = entry_path.to_string_lossy().into_owned();

            // Check reparse point / symlink / junction BEFORE recursing
            if is_reparse_point(&entry_path) {
                reparse_points_skipped += 1;
                continue;
            }

            let meta = match entry.metadata() {
                Ok(m) => m,
                Err(_) => {
                    errors_count += 1;
                    if sample_inaccessible_paths.len() < 50 {
                        sample_inaccessible_paths.push(entry_path_str);
                    }
                    if let Some(ref k) = imm_key {
                        if let Some(acc) = immediate_children_map.get_mut(k) {
                            acc.is_partial = true;
                        }
                    }
                    continue;
                }
            };

            if meta.is_dir() {
                folders_count += 1;
                let folder_name = entry.file_name().to_string_lossy().into_owned();

                let next_imm_key = match &imm_key {
                    None => {
                        immediate_children_map
                            .entry(entry_path_str.clone())
                            .or_insert_with(|| FolderAccum {
                                path: entry_path_str.clone(),
                                name: folder_name.clone(),
                                ..Default::default()
                            });
                        Some(entry_path_str.clone())
                    }
                    Some(existing_imm) => {
                        if let Some(acc) = immediate_children_map.get_mut(existing_imm) {
                            acc.folders_count += 1;
                        }
                        Some(existing_imm.clone())
                    }
                };

                let next_sec_key = if imm_key.is_some() && sec_key.is_none() {
                    second_level_folders_map
                        .entry(entry_path_str.clone())
                        .or_insert_with(|| FolderAccum {
                            path: entry_path_str.clone(),
                            name: folder_name,
                            ..Default::default()
                        });
                    Some(entry_path_str.clone())
                } else {
                    if let Some(ref existing_sec) = sec_key {
                        if let Some(acc) = second_level_folders_map.get_mut(existing_sec) {
                            acc.folders_count += 1;
                        }
                    }
                    sec_key.clone()
                };

                stack.push((entry_path, next_imm_key, next_sec_key));
            } else if meta.is_file() {
                files_count += 1;
                let logical_size = meta.len();
                let allocated_size = compute_allocated_size(&entry_path, logical_size);

                total_logical_bytes = total_logical_bytes.saturating_add(logical_size);
                total_allocated_bytes = total_allocated_bytes.saturating_add(allocated_size);

                match &imm_key {
                    Some(k) => {
                        if let Some(acc) = immediate_children_map.get_mut(k) {
                            acc.logical_size = acc.logical_size.saturating_add(logical_size);
                            acc.allocated_size = acc.allocated_size.saturating_add(allocated_size);
                            acc.files_count += 1;
                        }
                    }
                    None => {
                        root_files_accum.logical_size =
                            root_files_accum.logical_size.saturating_add(logical_size);
                        root_files_accum.allocated_size = root_files_accum
                            .allocated_size
                            .saturating_add(allocated_size);
                        root_files_accum.files_count += 1;
                    }
                }

                if let Some(ref k2) = sec_key {
                    if let Some(acc) = second_level_folders_map.get_mut(k2) {
                        acc.logical_size = acc.logical_size.saturating_add(logical_size);
                        acc.allocated_size = acc.allocated_size.saturating_add(allocated_size);
                        acc.files_count += 1;
                    }
                }

                let ext_lower = entry_path
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e.trim().to_ascii_lowercase())
                    .filter(|e| !e.is_empty())
                    .unwrap_or_else(|| "(no ext)".to_string());

                let category = classify_extension_category(&ext_lower).to_string();
                let ext_entry = ext_map
                    .entry(ext_lower.clone())
                    .or_insert_with(|| ExtAccum {
                        category: category.clone(),
                        ..Default::default()
                    });
                ext_entry.files_count += 1;
                ext_entry.logical_size = ext_entry.logical_size.saturating_add(logical_size);
                ext_entry.allocated_size = ext_entry.allocated_size.saturating_add(allocated_size);

                // Maintain bounded min-heap of top N largest files
                let should_insert = top_files_heap.len() < max_top_files
                    || top_files_heap
                        .peek()
                        .map(|smallest| logical_size > smallest.0.logical_size)
                        .unwrap_or(true);

                if should_insert {
                    let modified_at = meta
                        .modified()
                        .ok()
                        .map(|st| DateTime::<Utc>::from(st).to_rfc3339())
                        .unwrap_or_else(|| started_at.clone());
                    let file_name = entry.file_name().to_string_lossy().into_owned();
                    let parent_dir = curr_dir_str.clone();

                    top_files_heap.push(MinHeapFileItem(ScannedFileEntry {
                        path: entry_path_str,
                        file_name,
                        parent_dir,
                        extension: ext_lower,
                        category,
                        logical_size,
                        allocated_size,
                        modified_at,
                    }));
                    if top_files_heap.len() > max_top_files {
                        top_files_heap.pop();
                    }
                }
            }

            items_since_last_progress += 1;
            if items_since_last_progress >= 250 {
                items_since_last_progress = 0;
                on_progress(
                    files_count,
                    folders_count,
                    total_logical_bytes,
                    &curr_dir_str,
                    errors_count,
                );
            }
        }
    }

    on_progress(
        files_count,
        folders_count,
        total_logical_bytes,
        &normalized_root,
        errors_count,
    );

    let to_folder_summary = |acc: FolderAccum| -> ScannedFolderSummary {
        let pct = if total_logical_bytes > 0 {
            ((acc.logical_size as f64 / total_logical_bytes as f64) * 1000.0).round() / 10.0
        } else {
            0.0
        };
        ScannedFolderSummary {
            path: acc.path,
            folder_name: acc.name,
            logical_size: acc.logical_size,
            allocated_size: acc.allocated_size,
            files_count: acc.files_count,
            folders_count: acc.folders_count,
            is_partial: acc.is_partial,
            percentage_of_root: pct,
        }
    };

    let mut immediate_children: Vec<ScannedFolderSummary> = immediate_children_map
        .values()
        .cloned()
        .map(to_folder_summary)
        .collect();
    if root_files_accum.files_count > 0 {
        immediate_children.push(to_folder_summary(root_files_accum));
    }
    immediate_children.sort_by(|a, b| b.logical_size.cmp(&a.logical_size));

    let mut top_folders: Vec<ScannedFolderSummary> = immediate_children_map
        .into_values()
        .chain(second_level_folders_map.into_values())
        .map(to_folder_summary)
        .collect();
    top_folders.sort_by(|a, b| b.logical_size.cmp(&a.logical_size));
    top_folders.truncate(max_top_folders);

    let mut top_files: Vec<ScannedFileEntry> = top_files_heap.into_iter().map(|w| w.0).collect();
    top_files.sort_by(|a, b| b.logical_size.cmp(&a.logical_size));

    let mut category_map: HashMap<String, (u64, u64, u64)> = HashMap::new();
    let mut extension_breakdown: Vec<ExtensionStat> = ext_map
        .into_iter()
        .map(|(ext, acc)| {
            let cat_entry = category_map
                .entry(acc.category.clone())
                .or_insert((0, 0, 0));
            cat_entry.0 += acc.files_count;
            cat_entry.1 = cat_entry.1.saturating_add(acc.logical_size);
            cat_entry.2 = cat_entry.2.saturating_add(acc.allocated_size);

            let pct = if total_logical_bytes > 0 {
                ((acc.logical_size as f64 / total_logical_bytes as f64) * 1000.0).round() / 10.0
            } else {
                0.0
            };
            ExtensionStat {
                extension: ext,
                category: acc.category,
                files_count: acc.files_count,
                logical_size: acc.logical_size,
                allocated_size: acc.allocated_size,
                percentage: pct,
            }
        })
        .collect();
    extension_breakdown.sort_by(|a, b| b.logical_size.cmp(&a.logical_size));
    extension_breakdown.truncate(40);

    let mut category_breakdown: Vec<CategoryStat> = category_map
        .into_iter()
        .map(|(category, (count, log_sz, alloc_sz))| {
            let pct = if total_logical_bytes > 0 {
                ((log_sz as f64 / total_logical_bytes as f64) * 1000.0).round() / 10.0
            } else {
                0.0
            };
            CategoryStat {
                category,
                files_count: count,
                logical_size: log_sz,
                allocated_size: alloc_sz,
                percentage: pct,
            }
        })
        .collect();
    category_breakdown.sort_by(|a, b| b.logical_size.cmp(&a.logical_size));

    let is_partial = errors_count > 0;
    let status = if is_partial {
        "PARTIAL_COMPLETED".to_string()
    } else {
        "COMPLETED".to_string()
    };

    Ok(DiskScanResult {
        scan_id,
        target_path: normalized_root,
        status,
        started_at,
        completed_at: Utc::now().to_rfc3339(),
        duration_ms: timer.elapsed().as_millis() as u64,
        total_logical_bytes,
        total_allocated_bytes,
        files_count,
        folders_count,
        errors_count,
        reparse_points_skipped,
        is_partial,
        sample_inaccessible_paths,
        immediate_children,
        top_folders,
        top_files,
        extension_breakdown,
        category_breakdown,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_scan_directory_calculates_sizes_and_top_entries_accurately() {
        let dir = tempfile::tempdir().unwrap();
        let sub_a = dir.path().join("Games");
        let sub_b = dir.path().join("Videos");
        fs::create_dir_all(&sub_a).unwrap();
        fs::create_dir_all(&sub_b).unwrap();

        fs::write(sub_a.join("archive.pak"), vec![b'A'; 20_000]).unwrap();
        fs::write(sub_b.join("clip.mp4"), vec![b'B'; 50_000]).unwrap();
        fs::write(dir.path().join("readme.txt"), b"hello saleem").unwrap();

        let token = CancellationToken::new();
        let req = DiskScanRequest {
            target_path: dir.path().to_string_lossy().into_owned(),
            max_top_files: Some(10),
            max_top_folders: Some(10),
            exclude_patterns: None,
        };

        let result = scan_directory_with_progress(&req, &token, |_, _, _, _, _| {}).unwrap();
        assert_eq!(result.files_count, 3);
        assert_eq!(result.folders_count, 2);
        assert_eq!(result.total_logical_bytes, 20_000 + 50_000 + 12);
        assert!(result.total_allocated_bytes >= result.total_logical_bytes);
        assert!(!result.is_partial);
        assert_eq!(result.top_files[0].file_name, "clip.mp4");
        assert_eq!(result.top_files[0].logical_size, 50_000);
        assert_eq!(result.immediate_children[0].folder_name, "Videos");
    }

    #[test]
    fn test_scan_cancellation_stops_immediately() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("file1.bin"), vec![0u8; 1024]).unwrap();

        let token = CancellationToken::new();
        token.cancel();

        let req = DiskScanRequest {
            target_path: dir.path().to_string_lossy().into_owned(),
            max_top_files: None,
            max_top_folders: None,
            exclude_patterns: None,
        };

        let res = scan_directory_with_progress(&req, &token, |_, _, _, _, _| {});
        assert!(matches!(res, Err(SaleemError::ScanCancelled)));
    }

    #[test]
    fn test_reparse_point_junction_is_skipped_without_infinite_loop() {
        let dir = tempfile::tempdir().unwrap();
        let real_sub = dir.path().join("RealFolder");
        fs::create_dir_all(&real_sub).unwrap();
        fs::write(real_sub.join("data.bin"), vec![1u8; 4096]).unwrap();

        // Create a directory junction pointing back to `dir.path()` (which would infinite-loop if followed)
        let junction_path = real_sub.join("LoopJunction");
        let status = std::process::Command::new("cmd")
            .args([
                "/C",
                "mklink",
                "/J",
                &junction_path.to_string_lossy(),
                &dir.path().to_string_lossy(),
            ])
            .output();

        let token = CancellationToken::new();
        let req = DiskScanRequest {
            target_path: dir.path().to_string_lossy().into_owned(),
            max_top_files: None,
            max_top_folders: None,
            exclude_patterns: None,
        };

        let result = scan_directory_with_progress(&req, &token, |_, _, _, _, _| {}).unwrap();
        if let Ok(out) = status {
            if out.status.success() {
                assert!(
                    result.reparse_points_skipped >= 1,
                    "Junction must be detected and skipped"
                );
            }
        }
        assert_eq!(result.files_count, 1);
    }
}
