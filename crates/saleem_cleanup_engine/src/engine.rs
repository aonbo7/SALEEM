use crate::models::{
    CleanupCandidateItem, CleanupExecutionReport, CleanupExecutionRequest,
    CleanupItemExecutionOutcome, CleanupPlanResult, CleanupRuleSummary, CleanupScanRequest,
};
use crate::rules::{default_cleanup_rules, expand_windows_env_scope};
use chrono::{DateTime, Utc};
use saleem_core::paths::{
    assert_safe_for_modification, is_protected_system_path, normalize_lexical_windows_path,
};
use saleem_core::{CancellationToken, SaleemError, SaleemResult};
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime};
use uuid::Uuid;

/// Checks if a file is currently held open by any process on Windows by attempting
/// to open a read handle with `dwShareMode = 0` (exclusive sharing).
pub fn is_file_currently_locked(path: &Path) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::Foundation::{CloseHandle, GENERIC_READ, INVALID_HANDLE_VALUE};
        use windows_sys::Win32::Storage::FileSystem::{
            CreateFileW, FILE_ATTRIBUTE_NORMAL, OPEN_EXISTING,
        };

        let wide: Vec<u16> = path
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let handle = unsafe {
            CreateFileW(
                wide.as_ptr(),
                GENERIC_READ,
                0, // dwShareMode = 0: fails immediately if any process has an open handle
                std::ptr::null(),
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                std::ptr::null_mut(),
            )
        };

        if handle == INVALID_HANDLE_VALUE {
            return true;
        }
        unsafe {
            CloseHandle(handle);
        }
        false
    }

    #[cfg(not(windows))]
    {
        let _ = path;
        false
    }
}

#[cfg(windows)]
fn is_reparse_point(metadata: &std::fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0400;
    (metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT) != 0
}

#[cfg(not(windows))]
fn is_reparse_point(metadata: &std::fs::Metadata) -> bool {
    metadata.file_type().is_symlink()
}

fn compute_allocated_bytes(path: &Path, logical_size: u64) -> u64 {
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
            return ((high as u64) << 32) | (low as u64);
        }
    }
    const CLUSTER: u64 = 4096;
    ((logical_size + CLUSTER - 1) / CLUSTER) * CLUSTER
}

/// Executes Stages 1–4 of the 9-Stage Cleanup Pipeline:
/// `Scan -> Analyze -> Plan -> Preview`
pub fn scan_and_plan_cleanup_with_progress<F>(
    request: &CleanupScanRequest,
    cancel_token: &CancellationToken,
    mut on_progress: F,
) -> SaleemResult<CleanupPlanResult>
where
    F: FnMut(u64, u64, u64, &str, u64),
{
    let started_instant = Instant::now();
    let started_at = Utc::now().to_rfc3339();
    let plan_id = format!("clean-plan-{}", Uuid::new_v4());
    let max_candidates = request.max_candidates.unwrap_or(500).clamp(20, 3000);
    let now_sys = SystemTime::now();

    let all_rules = default_cleanup_rules();
    let mut rule_summaries = Vec::with_capacity(all_rules.len());
    let mut all_candidates = Vec::new();

    let mut files_seen: u64 = 0;
    let mut folders_seen: u64 = 0;
    let mut bytes_seen: u64 = 0;
    let mut errors_count: u64 = 0;
    let mut total_locked_skipped: usize = 0;
    let mut tick: u64 = 0;

    for rule in &all_rules {
        if cancel_token.is_cancelled() {
            return Err(SaleemError::ScanCancelled);
        }

        let is_enabled = match &request.enabled_rule_ids {
            Some(ids) => ids.iter().any(|id| id == &rule.id),
            None => rule.enabled_by_default,
        };

        let mut rule_candidates_count: usize = 0;
        let mut rule_locked_count: usize = 0;
        let mut rule_logical_bytes: u64 = 0;
        let mut rule_allocated_bytes: u64 = 0;

        if is_enabled {
            let scopes: Vec<PathBuf> = if let Some(sandbox) = &request.custom_sandbox_root {
                vec![PathBuf::from(sandbox).join(&rule.id)]
            } else {
                rule.allowed_scopes
                    .iter()
                    .filter_map(|s| expand_windows_env_scope(s))
                    .collect()
            };

            let min_age_hours = if request.custom_sandbox_root.is_some() {
                0
            } else {
                rule.min_age_hours
            };

            for scope_root in scopes {
                if !scope_root.exists() {
                    continue;
                }
                let scope_str = scope_root.to_string_lossy().into_owned();
                if is_protected_system_path(&scope_str) {
                    continue;
                }

                let mut stack = vec![scope_root];
                while let Some(curr_dir) = stack.pop() {
                    if cancel_token.is_cancelled() {
                        return Err(SaleemError::ScanCancelled);
                    }

                    let curr_str = curr_dir.to_string_lossy().into_owned();
                    if is_protected_system_path(&curr_str) {
                        continue;
                    }

                    let entries = match std::fs::read_dir(&curr_dir) {
                        Ok(rd) => rd,
                        Err(_) => {
                            errors_count += 1;
                            continue;
                        }
                    };

                    for entry_res in entries {
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
                        let meta = match std::fs::symlink_metadata(&entry_path) {
                            Ok(m) => m,
                            Err(_) => {
                                errors_count += 1;
                                continue;
                            }
                        };

                        if is_reparse_point(&meta) {
                            continue;
                        }

                        if meta.is_dir() {
                            folders_seen += 1;
                            stack.push(entry_path);
                        } else if meta.is_file() {
                            files_seen += 1;
                            let logical_size = meta.len();
                            if logical_size == 0 {
                                continue;
                            }

                            let file_name = entry.file_name().to_string_lossy().into_owned();
                            let file_name_lower = file_name.to_ascii_lowercase();

                            // Prefix filter check
                            if !rule.file_prefixes.is_empty()
                                && !rule.file_prefixes.iter().any(|pref| {
                                    file_name_lower.starts_with(&pref.to_ascii_lowercase())
                                })
                            {
                                continue;
                            }

                            // Extension filter check
                            if !rule.file_extensions.is_empty() {
                                let ext = entry_path
                                    .extension()
                                    .and_then(|e| e.to_str())
                                    .map(|e| e.to_ascii_lowercase())
                                    .unwrap_or_default();
                                if !rule.file_extensions.iter().any(|allowed| allowed == &ext) {
                                    continue;
                                }
                            }

                            // Age check
                            let modified_sys = meta.modified().unwrap_or(now_sys);
                            let age_secs = now_sys
                                .duration_since(modified_sys)
                                .map(|d| d.as_secs())
                                .unwrap_or(0);
                            let age_hours = age_secs / 3600;
                            if age_hours < min_age_hours {
                                continue;
                            }

                            let path_str = entry_path.to_string_lossy().into_owned();
                            if assert_safe_for_modification(&path_str).is_err() {
                                continue;
                            }

                            let locked =
                                rule.exclude_locked_files && is_file_currently_locked(&entry_path);
                            if locked {
                                rule_locked_count += 1;
                                total_locked_skipped += 1;
                                continue;
                            }

                            let allocated_size = compute_allocated_bytes(&entry_path, logical_size);
                            let modified_at = DateTime::<Utc>::from(modified_sys).to_rfc3339();

                            rule_candidates_count += 1;
                            rule_logical_bytes = rule_logical_bytes.saturating_add(logical_size);
                            rule_allocated_bytes =
                                rule_allocated_bytes.saturating_add(allocated_size);
                            bytes_seen = bytes_seen.saturating_add(logical_size);

                            all_candidates.push(CleanupCandidateItem {
                                candidate_id: format!("cand-{}-{}", rule.id, rule_candidates_count),
                                rule_id: rule.id.clone(),
                                rule_name_en: rule.name_en.clone(),
                                rule_name_ar: rule.name_ar.clone(),
                                category: rule.category.clone(),
                                risk_level: rule.risk_level.clone(),
                                path: path_str,
                                file_name,
                                parent_dir: curr_str.clone(),
                                logical_size,
                                allocated_size,
                                modified_at,
                                age_hours,
                                is_locked: false,
                                safe_to_clean: true,
                                reason_en: rule.reason_en.clone(),
                                reason_ar: rule.reason_ar.clone(),
                            });

                            tick += 1;
                            if tick >= 150 {
                                tick = 0;
                                on_progress(
                                    files_seen,
                                    folders_seen,
                                    bytes_seen,
                                    &curr_str,
                                    errors_count,
                                );
                            }
                        }
                    }
                }
            }
        }

        rule_summaries.push(CleanupRuleSummary {
            rule_id: rule.id.clone(),
            name_en: rule.name_en.clone(),
            name_ar: rule.name_ar.clone(),
            category: rule.category.clone(),
            risk_level: rule.risk_level.clone(),
            enabled_by_default: rule.enabled_by_default,
            candidates_count: rule_candidates_count,
            locked_skipped_count: rule_locked_count,
            total_logical_bytes: rule_logical_bytes,
            total_allocated_bytes: rule_allocated_bytes,
            reason_en: rule.reason_en.clone(),
            reason_ar: rule.reason_ar.clone(),
        });
    }

    // Sort candidates by largest size first so user sees most impactful items at top
    all_candidates.sort_by(|a, b| b.logical_size.cmp(&a.logical_size));

    let total_candidates = all_candidates.len();
    let total_reclaimable_bytes: u64 = all_candidates.iter().map(|c| c.logical_size).sum();
    let total_allocated_bytes: u64 = all_candidates.iter().map(|c| c.allocated_size).sum();

    if all_candidates.len() > max_candidates {
        all_candidates.truncate(max_candidates);
    }

    let completed_at = Utc::now().to_rfc3339();
    let duration_ms = started_instant.elapsed().as_millis() as u64;

    Ok(CleanupPlanResult {
        plan_id,
        pipeline_stage: "PreviewReady".to_string(),
        started_at,
        completed_at,
        duration_ms,
        total_candidates,
        total_locked_skipped,
        total_reclaimable_bytes,
        total_allocated_bytes,
        errors_count,
        rule_summaries,
        candidates: all_candidates,
    })
}

/// Executes Stages 5–9 of the 9-Stage Cleanup Pipeline:
/// `Confirm -> Validate -> Execute -> Verify -> Report`
pub fn validate_and_execute_cleanup(
    request: &CleanupExecutionRequest,
) -> SaleemResult<CleanupExecutionReport> {
    if request.selected_items.is_empty() {
        return Err(SaleemError::InvalidInput(
            "No cleanup items were selected for execution".to_string(),
        ));
    }

    let started_instant = Instant::now();
    let started_at = Utc::now().to_rfc3339();
    let job_id = format!("clean-job-{}", Uuid::new_v4());

    let mut succeeded_count: usize = 0;
    let mut skipped_or_failed_count: usize = 0;
    let mut bytes_reclaimed: u64 = 0;
    let mut outcomes = Vec::with_capacity(request.selected_items.len());

    for item in &request.selected_items {
        // Stage 6: Validate against Protected System Path Policy
        let normalized = match normalize_lexical_windows_path(&item.path) {
            Ok(p) => p,
            Err(e) => {
                skipped_or_failed_count += 1;
                outcomes.push(CleanupItemExecutionOutcome {
                    path: item.path.clone(),
                    rule_id: item.rule_id.clone(),
                    action: "DELETE_SAFE_CANDIDATE".to_string(),
                    status: "REJECTED_POLICY".to_string(),
                    bytes_reclaimed: 0,
                    detail: e.to_string(),
                });
                continue;
            }
        };

        let safe_path = match assert_safe_for_modification(&normalized) {
            Ok(p) => p,
            Err(e) => {
                skipped_or_failed_count += 1;
                outcomes.push(CleanupItemExecutionOutcome {
                    path: normalized,
                    rule_id: item.rule_id.clone(),
                    action: "DELETE_SAFE_CANDIDATE".to_string(),
                    status: "REJECTED_POLICY".to_string(),
                    bytes_reclaimed: 0,
                    detail: format!("Blocked by Protected System Path Policy: {}", e),
                });
                continue;
            }
        };

        // Ensure file exists and is a regular file (not a directory or symlink)
        let meta = match std::fs::symlink_metadata(&safe_path) {
            Ok(m) => m,
            Err(e) => {
                skipped_or_failed_count += 1;
                outcomes.push(CleanupItemExecutionOutcome {
                    path: normalized,
                    rule_id: item.rule_id.clone(),
                    action: "DELETE_SAFE_CANDIDATE".to_string(),
                    status: "FAILED_IO".to_string(),
                    bytes_reclaimed: 0,
                    detail: format!("Could not stat file prior to deletion: {}", e),
                });
                continue;
            }
        };

        if !meta.is_file() || is_reparse_point(&meta) {
            skipped_or_failed_count += 1;
            outcomes.push(CleanupItemExecutionOutcome {
                path: normalized,
                rule_id: item.rule_id.clone(),
                action: "DELETE_SAFE_CANDIDATE".to_string(),
                status: "REJECTED_POLICY".to_string(),
                bytes_reclaimed: 0,
                detail: "Target is not a regular file or is a reparse point".to_string(),
            });
            continue;
        }

        // Pre-action size sanity check (TOCTOU guard)
        let current_size = meta.len();
        if item.expected_size_bytes > 0 && current_size != item.expected_size_bytes {
            skipped_or_failed_count += 1;
            outcomes.push(CleanupItemExecutionOutcome {
                path: normalized,
                rule_id: item.rule_id.clone(),
                action: "DELETE_SAFE_CANDIDATE".to_string(),
                status: "SKIPPED_MODIFIED".to_string(),
                bytes_reclaimed: 0,
                detail: format!(
                    "File size changed since preview (expected {} B, found {} B)",
                    item.expected_size_bytes, current_size
                ),
            });
            continue;
        }

        // Check if file became locked by an active process
        if is_file_currently_locked(&safe_path) {
            skipped_or_failed_count += 1;
            outcomes.push(CleanupItemExecutionOutcome {
                path: normalized,
                rule_id: item.rule_id.clone(),
                action: "DELETE_SAFE_CANDIDATE".to_string(),
                status: "SKIPPED_LOCKED".to_string(),
                bytes_reclaimed: 0,
                detail: "File is currently locked by an active process; skipped safely".to_string(),
            });
            continue;
        }

        // Stage 7: Execute deletion
        match std::fs::remove_file(&safe_path) {
            Ok(()) => {
                // Stage 8: Post-execution verification (`!safe_path.exists()`)
                if !safe_path.exists() {
                    succeeded_count += 1;
                    bytes_reclaimed = bytes_reclaimed.saturating_add(current_size);
                    outcomes.push(CleanupItemExecutionOutcome {
                        path: normalized,
                        rule_id: item.rule_id.clone(),
                        action: "DELETE_SAFE_CANDIDATE".to_string(),
                        status: "VERIFIED_REMOVED".to_string(),
                        bytes_reclaimed: current_size,
                        detail: "Removed and verified absent on disk".to_string(),
                    });
                } else {
                    skipped_or_failed_count += 1;
                    outcomes.push(CleanupItemExecutionOutcome {
                        path: normalized,
                        rule_id: item.rule_id.clone(),
                        action: "DELETE_SAFE_CANDIDATE".to_string(),
                        status: "FAILED_IO".to_string(),
                        bytes_reclaimed: 0,
                        detail: "remove_file returned Ok but path still exists".to_string(),
                    });
                }
            }
            Err(e) => {
                skipped_or_failed_count += 1;
                outcomes.push(CleanupItemExecutionOutcome {
                    path: normalized,
                    rule_id: item.rule_id.clone(),
                    action: "DELETE_SAFE_CANDIDATE".to_string(),
                    status: "FAILED_IO".to_string(),
                    bytes_reclaimed: 0,
                    detail: format!("OS remove_file error: {}", e),
                });
            }
        }
    }

    let completed_at = Utc::now().to_rfc3339();
    let duration_ms = started_instant.elapsed().as_millis() as u64;

    // Stage 9: Report
    Ok(CleanupExecutionReport {
        job_id,
        plan_id: request.plan_id.clone(),
        pipeline_stage: "CompletedAndVerified".to_string(),
        started_at,
        completed_at,
        duration_ms,
        total_requested: request.selected_items.len(),
        succeeded_count,
        skipped_or_failed_count,
        bytes_reclaimed,
        outcomes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::CleanupExecutionItemRequest;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_cleanup_scan_and_plan_discovers_candidates_and_respects_rules() {
        let sandbox = tempfile::tempdir().unwrap();
        let temp_rule_dir = sandbox.path().join("CLEAN-TEMP-USER-001");
        std::fs::create_dir_all(&temp_rule_dir).unwrap();

        let file_a = temp_rule_dir.join("stale_scratch.tmp");
        std::fs::write(&file_a, vec![b'X'; 4096]).unwrap();

        let req = CleanupScanRequest {
            enabled_rule_ids: Some(vec!["CLEAN-TEMP-USER-001".to_string()]),
            custom_sandbox_root: Some(sandbox.path().to_string_lossy().into_owned()),
            max_candidates: Some(100),
        };
        let token = CancellationToken::new();
        let plan = scan_and_plan_cleanup_with_progress(&req, &token, |_, _, _, _, _| {}).unwrap();

        assert_eq!(plan.pipeline_stage, "PreviewReady");
        assert_eq!(plan.total_candidates, 1);
        assert_eq!(plan.total_reclaimable_bytes, 4096);
        assert_eq!(plan.candidates[0].rule_id, "CLEAN-TEMP-USER-001");
    }

    #[test]
    fn test_locked_file_is_detected_and_never_deleted() {
        let sandbox = tempfile::tempdir().unwrap();
        let locked_path = sandbox.path().join("active_in_use.tmp");

        let mut open_handle = File::create(&locked_path).unwrap();
        open_handle.write_all(&vec![b'L'; 2048]).unwrap();
        open_handle.flush().unwrap();

        // While `open_handle` is alive, `is_file_currently_locked` must return true on Windows
        assert!(is_file_currently_locked(&locked_path));

        let exec_req = CleanupExecutionRequest {
            plan_id: "plan-lock-test".to_string(),
            selected_items: vec![CleanupExecutionItemRequest {
                path: locked_path.to_string_lossy().into_owned(),
                rule_id: "CLEAN-TEMP-USER-001".to_string(),
                expected_size_bytes: 2048,
            }],
        };

        let report = validate_and_execute_cleanup(&exec_req).unwrap();
        assert_eq!(report.succeeded_count, 0);
        assert_eq!(report.skipped_or_failed_count, 1);
        assert_eq!(report.outcomes[0].status, "SKIPPED_LOCKED");
        assert!(locked_path.exists());
        drop(open_handle);
    }

    #[test]
    fn test_cleanup_execution_rejects_protected_system_paths() {
        let exec_req = CleanupExecutionRequest {
            plan_id: "plan-policy-test".to_string(),
            selected_items: vec![
                CleanupExecutionItemRequest {
                    path: r"C:\Windows\System32\kernel32.dll".to_string(),
                    rule_id: "CLEAN-TEMP-USER-001".to_string(),
                    expected_size_bytes: 1024,
                },
                CleanupExecutionItemRequest {
                    path: r"C:\Windows\WinSxS\manifest.xml".to_string(),
                    rule_id: "CLEAN-TEMP-USER-001".to_string(),
                    expected_size_bytes: 1024,
                },
            ],
        };

        let report = validate_and_execute_cleanup(&exec_req).unwrap();
        assert_eq!(report.succeeded_count, 0);
        assert_eq!(report.skipped_or_failed_count, 2);
        assert!(report
            .outcomes
            .iter()
            .all(|o| o.status == "REJECTED_POLICY"));
    }

    #[test]
    fn test_cleanup_execute_verifies_removal_and_reports_accurate_reclaimed_bytes() {
        let sandbox = tempfile::tempdir().unwrap();
        let target = sandbox.path().join("safe_to_remove.tmp");
        std::fs::write(&target, vec![b'Z'; 8192]).unwrap();

        let exec_req = CleanupExecutionRequest {
            plan_id: "plan-verify-test".to_string(),
            selected_items: vec![CleanupExecutionItemRequest {
                path: target.to_string_lossy().into_owned(),
                rule_id: "CLEAN-TEMP-USER-001".to_string(),
                expected_size_bytes: 8192,
            }],
        };

        let report = validate_and_execute_cleanup(&exec_req).unwrap();
        assert_eq!(report.succeeded_count, 1);
        assert_eq!(report.bytes_reclaimed, 8192);
        assert_eq!(report.outcomes[0].status, "VERIFIED_REMOVED");
        assert!(!target.exists());
    }
}
