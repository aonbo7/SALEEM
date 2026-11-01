use crate::ffi_loader::{inspect_engine_dll, NativeDllStatus};
use crate::state::AppState;
use saleem_cleanup_engine::{
    default_cleanup_rules, scan_and_plan_cleanup_with_progress, validate_and_execute_cleanup,
    CleanupExecutionItemRequest, CleanupExecutionReport, CleanupExecutionRequest,
    CleanupPlanResult, CleanupRuleDefinition, CleanupScanRequest,
};
use saleem_core::db::{
    get_database_health_summary, get_setting, list_recent_audit_events, record_audit_event,
    set_setting, AuditEventRecord, DatabaseHealthSummary,
};
use saleem_core::paths::normalize_lexical_windows_path;
use saleem_core::{JobProgressSnapshot, JobStatus, SaleemError, UserErrorPayload};
use saleem_disk_engine::{
    find_duplicates_with_progress, list_volumes, scan_directory_with_progress,
    verify_and_remove_duplicate_selections, DiskScanRequest, DiskScanResult,
    DuplicateRemovalGroupSelection, DuplicateRemovalReport, DuplicateScanRequest,
    DuplicateScanResult, VolumeInfo,
};
use saleem_inventory::{
    remove_verified_leftovers, scan_inventory_and_leftovers, InventoryScanRequest,
    InventoryScanResult, LeftoverRemovalItemRequest, LeftoverRemovalReport, LeftoverRemovalRequest,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemFoundationStatus {
    pub app_name: String,
    pub app_version: String,
    pub current_phase: String,
    pub ui_language: String,
    pub ui_theme: String,
    pub database: DatabaseHealthSummary,
    pub engines: Vec<NativeDllStatus>,
    pub volumes: Vec<VolumeInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickScanPreset {
    pub id: String,
    pub label_en: String,
    pub label_ar: String,
    pub path: String,
}

fn collect_engines_status() -> Vec<NativeDllStatus> {
    vec![
        inspect_engine_dll(
            "SALEEM Disk Engine",
            "SALEEM.DiskEngine.dll",
            "saleem_disk_engine.dll",
            b"disk_engine_abi_version\0",
            saleem_disk_engine::disk_engine_abi_version,
        ),
        inspect_engine_dll(
            "SALEEM Cleanup Engine",
            "SALEEM.CleanupEngine.dll",
            "saleem_cleanup_engine.dll",
            b"cleanup_engine_abi_version\0",
            saleem_cleanup_engine::cleanup_engine_abi_version,
        ),
        inspect_engine_dll(
            "SALEEM Sentinel",
            "SALEEM.Sentinel.dll",
            "saleem_sentinel.dll",
            b"sentinel_abi_version\0",
            saleem_sentinel::sentinel_abi_version,
        ),
        inspect_engine_dll(
            "SALEEM Memory Engine (MemReduct)",
            "SALEEM.MemoryEngine.dll",
            "saleem_memory_engine.dll",
            b"memory_engine_abi_version\0",
            saleem_memory_engine::memory_engine_abi_version,
        ),
    ]
}

#[tauri::command]
pub fn get_system_foundation_status(
    state: State<'_, AppState>,
) -> Result<SystemFoundationStatus, UserErrorPayload> {
    let (db_summary, ui_language, ui_theme) = state
        .with_db(|conn| {
            let summary = get_database_health_summary(conn)?;
            let lang = get_setting(conn, "ui.language")?.unwrap_or_else(|| "ar".to_string());
            let theme = get_setting(conn, "ui.theme")?.unwrap_or_else(|| "dark".to_string());
            Ok((summary, lang, theme))
        })
        .map_err(|e| e.to_user_payload())?;

    let volumes = list_volumes().map_err(|e| e.to_user_payload())?;
    let engines = collect_engines_status();

    Ok(SystemFoundationStatus {
        app_name: "SALEEM".to_string(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        current_phase: "Phase 4 — App, Game & Leftover Inventory + Crystal Glass UI".to_string(),
        ui_language,
        ui_theme,
        database: db_summary,
        engines,
        volumes,
    })
}

#[tauri::command]
pub fn get_volumes_command() -> Result<Vec<VolumeInfo>, UserErrorPayload> {
    list_volumes().map_err(|e| e.to_user_payload())
}

#[tauri::command]
pub fn get_quick_scan_presets_command() -> Result<Vec<QuickScanPreset>, UserErrorPayload> {
    let mut presets = Vec::new();
    if let Ok(user_profile) = std::env::var("USERPROFILE") {
        let up = PathBuf::from(&user_profile);
        if up.exists() {
            presets.push(QuickScanPreset {
                id: "user_profile".into(),
                label_en: "User Home".into(),
                label_ar: "مجلد المستخدم".into(),
                path: user_profile.clone(),
            });
        }
        let downloads = up.join("Downloads");
        if downloads.exists() {
            presets.push(QuickScanPreset {
                id: "downloads".into(),
                label_en: "Downloads".into(),
                label_ar: "التنزيلات".into(),
                path: downloads.to_string_lossy().into_owned(),
            });
        }
        let documents = up.join("Documents");
        if documents.exists() {
            presets.push(QuickScanPreset {
                id: "documents".into(),
                label_en: "Documents".into(),
                label_ar: "المستندات".into(),
                path: documents.to_string_lossy().into_owned(),
            });
        }
    }
    if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
        let p = PathBuf::from(&local_appdata);
        if p.exists() {
            presets.push(QuickScanPreset {
                id: "local_appdata".into(),
                label_en: "AppData Local".into(),
                label_ar: "بيانات التطبيقات (Local)".into(),
                path: local_appdata,
            });
        }
    }
    if let Ok(vols) = list_volumes() {
        for v in vols.into_iter().filter(|v| v.is_ready) {
            let label = if v.volume_label.is_empty() {
                v.mount_point.clone()
            } else {
                format!("{} ({})", v.mount_point, v.volume_label)
            };
            presets.push(QuickScanPreset {
                id: format!("vol_{}", v.mount_point),
                label_en: format!("Drive {}", label),
                label_ar: format!("قرص {}", label),
                path: v.mount_point,
            });
        }
    }
    Ok(presets)
}

#[tauri::command]
pub fn start_disk_scan_command(
    state: State<'_, AppState>,
    target_path: String,
    max_top_files: Option<usize>,
    max_top_folders: Option<usize>,
) -> Result<String, UserErrorPayload> {
    let normalized =
        normalize_lexical_windows_path(&target_path).map_err(|e| e.to_user_payload())?;
    if !PathBuf::from(&normalized).exists() {
        return Err(SaleemError::PathNotFound(normalized).to_user_payload());
    }

    let (job_id, token) = state.job_registry.create_job("DISK_SCAN", &normalized);
    let state_clone = state.inner().clone();
    let job_id_clone = job_id.clone();

    std::thread::spawn(move || {
        let req = DiskScanRequest {
            target_path: normalized,
            max_top_files: Some(max_top_files.unwrap_or(250)),
            max_top_folders: Some(max_top_folders.unwrap_or(100)),
            exclude_patterns: None,
        };

        let registry_ref = state_clone.job_registry.clone();
        let jid = job_id_clone.clone();

        let outcome =
            scan_directory_with_progress(&req, &token, |files, folders, bytes, curr, errs| {
                registry_ref.update_progress(&jid, files, folders, bytes, curr, errs);
            });

        match outcome {
            Ok(result) => {
                let _ = state_clone.persist_disk_scan(&result);
                if let Ok(mut map) = state_clone.scan_results.lock() {
                    map.insert(job_id_clone.clone(), result);
                }
                if let Ok(mut latest) = state_clone.latest_scan_job_id.lock() {
                    *latest = Some(job_id_clone.clone());
                }
                state_clone
                    .job_registry
                    .finish_job(&job_id_clone, JobStatus::Completed);
            }
            Err(SaleemError::ScanCancelled) => {
                let _ = state_clone.with_db(|conn| {
                    record_audit_event(
                        conn,
                        "DISK_SCAN_CANCELLED",
                        "SALEEM.DiskEngine",
                        "CANCELLED",
                        &format!("job_id={}", job_id_clone),
                    )
                });
                state_clone
                    .job_registry
                    .finish_job(&job_id_clone, JobStatus::Cancelled);
            }
            Err(_) => {
                state_clone
                    .job_registry
                    .finish_job(&job_id_clone, JobStatus::Failed);
            }
        }
    });

    Ok(job_id)
}

#[tauri::command]
pub fn get_job_progress_command(
    state: State<'_, AppState>,
    job_id: String,
) -> Result<JobProgressSnapshot, UserErrorPayload> {
    state.job_registry.get_snapshot(&job_id).ok_or_else(|| {
        SaleemError::InvalidInput(format!("Unknown job_id: {}", job_id)).to_user_payload()
    })
}

#[tauri::command]
pub fn cancel_job_command(
    state: State<'_, AppState>,
    job_id: String,
) -> Result<bool, UserErrorPayload> {
    Ok(state.job_registry.cancel_job(&job_id))
}

#[tauri::command]
pub fn get_disk_scan_result_command(
    state: State<'_, AppState>,
    job_id: String,
) -> Result<Option<DiskScanResult>, UserErrorPayload> {
    let res = state
        .scan_results
        .lock()
        .map_err(|_| SaleemError::IoError("Scan results lock poisoned".into()).to_user_payload())?
        .get(&job_id)
        .cloned();
    Ok(res)
}

#[tauri::command]
pub fn get_latest_disk_scan_command(
    state: State<'_, AppState>,
) -> Result<Option<DiskScanResult>, UserErrorPayload> {
    let latest_id = state
        .latest_scan_job_id
        .lock()
        .map_err(|_| SaleemError::IoError("Lock poisoned".into()).to_user_payload())?
        .clone();
    if let Some(jid) = latest_id {
        get_disk_scan_result_command(state, jid)
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub fn start_duplicate_scan_command(
    state: State<'_, AppState>,
    target_path: String,
    min_size_bytes: Option<u64>,
    max_groups: Option<usize>,
) -> Result<String, UserErrorPayload> {
    let normalized =
        normalize_lexical_windows_path(&target_path).map_err(|e| e.to_user_payload())?;
    if !PathBuf::from(&normalized).exists() {
        return Err(SaleemError::PathNotFound(normalized).to_user_payload());
    }

    let (job_id, token) = state.job_registry.create_job("DUPLICATE_SCAN", &normalized);
    let state_clone = state.inner().clone();
    let job_id_clone = job_id.clone();

    std::thread::spawn(move || {
        let req = DuplicateScanRequest {
            target_path: normalized,
            min_file_size_bytes: Some(min_size_bytes.unwrap_or(1024)),
            max_groups: Some(max_groups.unwrap_or(150)),
        };

        let registry_ref = state_clone.job_registry.clone();
        let jid = job_id_clone.clone();

        let outcome =
            find_duplicates_with_progress(&req, &token, |files, folders, bytes, curr, errs| {
                registry_ref.update_progress(&jid, files, folders, bytes, curr, errs);
            });

        match outcome {
            Ok(result) => {
                let _ = state_clone.persist_duplicate_scan(&result);
                if let Ok(mut map) = state_clone.duplicate_results.lock() {
                    map.insert(job_id_clone.clone(), result);
                }
                if let Ok(mut latest) = state_clone.latest_duplicate_job_id.lock() {
                    *latest = Some(job_id_clone.clone());
                }
                state_clone
                    .job_registry
                    .finish_job(&job_id_clone, JobStatus::Completed);
            }
            Err(SaleemError::ScanCancelled) => {
                let _ = state_clone.with_db(|conn| {
                    record_audit_event(
                        conn,
                        "DUPLICATE_SCAN_CANCELLED",
                        "SALEEM.DiskEngine",
                        "CANCELLED",
                        &format!("job_id={}", job_id_clone),
                    )
                });
                state_clone
                    .job_registry
                    .finish_job(&job_id_clone, JobStatus::Cancelled);
            }
            Err(_) => {
                state_clone
                    .job_registry
                    .finish_job(&job_id_clone, JobStatus::Failed);
            }
        }
    });

    Ok(job_id)
}

#[tauri::command]
pub fn get_duplicate_scan_result_command(
    state: State<'_, AppState>,
    job_id: String,
) -> Result<Option<DuplicateScanResult>, UserErrorPayload> {
    let res = state
        .duplicate_results
        .lock()
        .map_err(|_| {
            SaleemError::IoError("Duplicate results lock poisoned".into()).to_user_payload()
        })?
        .get(&job_id)
        .cloned();
    Ok(res)
}

#[tauri::command]
pub fn get_latest_duplicate_scan_command(
    state: State<'_, AppState>,
) -> Result<Option<DuplicateScanResult>, UserErrorPayload> {
    let latest_id = state
        .latest_duplicate_job_id
        .lock()
        .map_err(|_| SaleemError::IoError("Lock poisoned".into()).to_user_payload())?
        .clone();
    if let Some(jid) = latest_id {
        get_duplicate_scan_result_command(state, jid)
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub fn execute_duplicate_removal_command(
    state: State<'_, AppState>,
    selections: Vec<DuplicateRemovalGroupSelection>,
) -> Result<DuplicateRemovalReport, UserErrorPayload> {
    let report =
        verify_and_remove_duplicate_selections(&selections).map_err(|e| e.to_user_payload())?;
    let _ = state.persist_duplicate_removal(&selections, &report);

    // Prune removed paths from the cached latest duplicate scan result so the UI reflects real-time state
    if !report.deleted_paths.is_empty() {
        if let Ok(mut map) = state.duplicate_results.lock() {
            for scan_res in map.values_mut() {
                for group in &mut scan_res.groups {
                    group.copies.retain(|m| {
                        !report
                            .deleted_paths
                            .iter()
                            .any(|rem| rem.eq_ignore_ascii_case(&m.path))
                    });
                    let distinct_files =
                        group.copies.iter().filter(|m| !m.is_hardlink_alias).count();
                    group.wasted_bytes = if distinct_files > 1 {
                        (distinct_files as u64 - 1) * group.file_size_bytes
                    } else {
                        0
                    };
                }
                scan_res.groups.retain(|g| g.copies.len() >= 2);
                scan_res.total_duplicate_groups = scan_res.groups.len();
                scan_res.total_duplicate_files =
                    scan_res.groups.iter().map(|g| g.copies.len()).sum();
                scan_res.total_reclaimable_bytes =
                    scan_res.groups.iter().map(|g| g.wasted_bytes).sum();
            }
        }
    }

    Ok(report)
}

#[tauri::command]
pub fn get_cleanup_rules_command() -> Result<Vec<CleanupRuleDefinition>, UserErrorPayload> {
    Ok(default_cleanup_rules())
}

#[tauri::command]
pub fn start_cleanup_scan_command(
    state: State<'_, AppState>,
    enabled_rule_ids: Option<Vec<String>>,
) -> Result<String, UserErrorPayload> {
    let (job_id, token) = state
        .job_registry
        .create_job("SAFE_CLEANUP_SCAN", "User Temp & Cache Scopes");
    let state_clone = state.inner().clone();
    let job_id_clone = job_id.clone();

    std::thread::spawn(move || {
        let req = CleanupScanRequest {
            enabled_rule_ids,
            custom_sandbox_root: None,
            max_candidates: Some(600),
        };

        let registry_ref = state_clone.job_registry.clone();
        let jid = job_id_clone.clone();

        let outcome = scan_and_plan_cleanup_with_progress(
            &req,
            &token,
            |files, folders, bytes, curr, errs| {
                registry_ref.update_progress(&jid, files, folders, bytes, curr, errs);
            },
        );

        match outcome {
            Ok(plan) => {
                let _ = state_clone.persist_cleanup_plan(&plan);
                if let Ok(mut map) = state_clone.cleanup_plans.lock() {
                    map.insert(job_id_clone.clone(), plan);
                }
                if let Ok(mut latest) = state_clone.latest_cleanup_job_id.lock() {
                    *latest = Some(job_id_clone.clone());
                }
                state_clone
                    .job_registry
                    .finish_job(&job_id_clone, JobStatus::Completed);
            }
            Err(SaleemError::ScanCancelled) => {
                let _ = state_clone.with_db(|conn| {
                    record_audit_event(
                        conn,
                        "CLEANUP_SCAN_CANCELLED",
                        "SALEEM.CleanupEngine",
                        "CANCELLED",
                        &format!("job_id={}", job_id_clone),
                    )
                });
                state_clone
                    .job_registry
                    .finish_job(&job_id_clone, JobStatus::Cancelled);
            }
            Err(_) => {
                state_clone
                    .job_registry
                    .finish_job(&job_id_clone, JobStatus::Failed);
            }
        }
    });

    Ok(job_id)
}

#[tauri::command]
pub fn get_cleanup_plan_result_command(
    state: State<'_, AppState>,
    job_id: String,
) -> Result<Option<CleanupPlanResult>, UserErrorPayload> {
    let res = state
        .cleanup_plans
        .lock()
        .map_err(|_| SaleemError::IoError("Cleanup plans lock poisoned".into()).to_user_payload())?
        .get(&job_id)
        .cloned();
    Ok(res)
}

#[tauri::command]
pub fn get_latest_cleanup_plan_command(
    state: State<'_, AppState>,
) -> Result<Option<CleanupPlanResult>, UserErrorPayload> {
    let latest_id = state
        .latest_cleanup_job_id
        .lock()
        .map_err(|_| SaleemError::IoError("Lock poisoned".into()).to_user_payload())?
        .clone();
    if let Some(jid) = latest_id {
        get_cleanup_plan_result_command(state, jid)
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub fn execute_cleanup_plan_command(
    state: State<'_, AppState>,
    plan_id: String,
    selected_items: Vec<CleanupExecutionItemRequest>,
) -> Result<CleanupExecutionReport, UserErrorPayload> {
    let req = CleanupExecutionRequest {
        plan_id,
        selected_items,
    };
    let report = validate_and_execute_cleanup(&req).map_err(|e| e.to_user_payload())?;
    let _ = state.persist_cleanup_execution(&report);

    let removed_paths: Vec<String> = report
        .outcomes
        .iter()
        .filter(|o| o.status == "VERIFIED_REMOVED")
        .map(|o| o.path.clone())
        .collect();

    if !removed_paths.is_empty() {
        // Prune removed candidates from cached cleanup plans
        if let Ok(mut map) = state.cleanup_plans.lock() {
            for plan in map.values_mut() {
                plan.candidates.retain(|c| {
                    !removed_paths
                        .iter()
                        .any(|rem| rem.eq_ignore_ascii_case(&c.path))
                });
                plan.total_candidates = plan.candidates.len();
                plan.total_reclaimable_bytes = plan.candidates.iter().map(|c| c.logical_size).sum();
                plan.total_allocated_bytes = plan.candidates.iter().map(|c| c.allocated_size).sum();

                for summary in &mut plan.rule_summaries {
                    let rule_cands: Vec<&saleem_cleanup_engine::CleanupCandidateItem> = plan
                        .candidates
                        .iter()
                        .filter(|c| c.rule_id == summary.rule_id)
                        .collect();
                    summary.candidates_count = rule_cands.len();
                    summary.total_logical_bytes = rule_cands.iter().map(|c| c.logical_size).sum();
                    summary.total_allocated_bytes =
                        rule_cands.iter().map(|c| c.allocated_size).sum();
                }
            }
        }

        // Also prune removed files from cached DiskScanResult.top_files (when user removes Large Files!)
        if let Ok(mut scan_map) = state.scan_results.lock() {
            for scan in scan_map.values_mut() {
                scan.top_files.retain(|f| {
                    !removed_paths
                        .iter()
                        .any(|rem| rem.eq_ignore_ascii_case(&f.path))
                });
            }
        }
    }

    Ok(report)
}

#[tauri::command]
pub fn start_inventory_scan_command(
    state: State<'_, AppState>,
) -> Result<String, UserErrorPayload> {
    let (job_id, token) = state.job_registry.create_job(
        "APP_AND_LEFTOVER_INVENTORY",
        "Registry + Launchers + AppData",
    );
    let state_clone = state.inner().clone();
    let job_id_clone = job_id.clone();

    std::thread::spawn(move || {
        let req = InventoryScanRequest {
            custom_sandbox_root: None,
            mock_installed_apps: None,
        };

        state_clone.job_registry.update_progress(
            &job_id_clone,
            0,
            0,
            0,
            "Scanning Windows Registry Uninstall Hives & Game Libraries...",
            0,
        );

        let outcome = scan_inventory_and_leftovers(&req, &token);

        match outcome {
            Ok(result) => {
                state_clone.job_registry.update_progress(
                    &job_id_clone,
                    (result.total_apps_count + result.total_games_count) as u64,
                    result.total_leftovers_count as u64,
                    result.total_leftover_bytes,
                    "Inventory & Orphaned Leftover Scan Complete",
                    0,
                );
                let _ = state_clone.persist_inventory_scan(&result);
                if let Ok(mut map) = state_clone.inventory_results.lock() {
                    map.insert(job_id_clone.clone(), result);
                }
                if let Ok(mut latest) = state_clone.latest_inventory_job_id.lock() {
                    *latest = Some(job_id_clone.clone());
                }
                state_clone
                    .job_registry
                    .finish_job(&job_id_clone, JobStatus::Completed);
            }
            Err(SaleemError::ScanCancelled) => {
                let _ = state_clone.with_db(|conn| {
                    record_audit_event(
                        conn,
                        "INVENTORY_SCAN_CANCELLED",
                        "SALEEM.Inventory",
                        "CANCELLED",
                        &format!("job_id={}", job_id_clone),
                    )
                });
                state_clone
                    .job_registry
                    .finish_job(&job_id_clone, JobStatus::Cancelled);
            }
            Err(_) => {
                state_clone
                    .job_registry
                    .finish_job(&job_id_clone, JobStatus::Failed);
            }
        }
    });

    Ok(job_id)
}

#[tauri::command]
pub fn get_inventory_scan_result_command(
    state: State<'_, AppState>,
    job_id: String,
) -> Result<Option<InventoryScanResult>, UserErrorPayload> {
    let res = state
        .inventory_results
        .lock()
        .map_err(|_| {
            SaleemError::IoError("Inventory results lock poisoned".into()).to_user_payload()
        })?
        .get(&job_id)
        .cloned();
    Ok(res)
}

#[tauri::command]
pub fn get_latest_inventory_result_command(
    state: State<'_, AppState>,
) -> Result<Option<InventoryScanResult>, UserErrorPayload> {
    let latest_id = state
        .latest_inventory_job_id
        .lock()
        .map_err(|_| SaleemError::IoError("Lock poisoned".into()).to_user_payload())?
        .clone();
    if let Some(jid) = latest_id {
        get_inventory_scan_result_command(state, jid)
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub fn execute_leftover_removal_command(
    state: State<'_, AppState>,
    scan_id: String,
    items: Vec<LeftoverRemovalItemRequest>,
) -> Result<LeftoverRemovalReport, UserErrorPayload> {
    let req = LeftoverRemovalRequest {
        scan_id,
        items,
        user_confirmed: true,
        custom_sandbox_root: None,
    };

    let report = remove_verified_leftovers(&req).map_err(|e| e.to_user_payload())?;
    let _ = state.persist_leftover_removal(&report);

    let removed_paths: Vec<String> = report
        .outcomes
        .iter()
        .filter(|o| o.status == "REMOVED_VERIFIED")
        .map(|o| o.path.clone())
        .collect();

    if !removed_paths.is_empty() {
        if let Ok(mut map) = state.inventory_results.lock() {
            for inv in map.values_mut() {
                inv.leftover_candidates.retain(|c| {
                    !removed_paths
                        .iter()
                        .any(|rem| rem.eq_ignore_ascii_case(&c.path))
                });
                inv.total_leftovers_count = inv.leftover_candidates.len();
                inv.high_confidence_leftovers_count = inv
                    .leftover_candidates
                    .iter()
                    .filter(|c| c.confidence == "High")
                    .count();
                inv.total_leftover_bytes = inv
                    .leftover_candidates
                    .iter()
                    .map(|c| c.logical_size_bytes)
                    .sum();
                inv.high_confidence_reclaimable_bytes = inv
                    .leftover_candidates
                    .iter()
                    .filter(|c| c.confidence == "High")
                    .map(|c| c.logical_size_bytes)
                    .sum();
            }
        }
    }

    Ok(report)
}

#[tauri::command]
pub fn launch_app_uninstaller_command(
    state: State<'_, AppState>,
    app_name: String,
    source: String,
    uninstall_string: Option<String>,
) -> Result<String, UserErrorPayload> {
    #[cfg(windows)]
    use std::os::windows::process::CommandExt;

    let raw_cmd = uninstall_string.unwrap_or_default().trim().to_string();

    if raw_cmd.to_ascii_lowercase().starts_with("steam://uninstall/") {
        std::process::Command::new("explorer.exe")
            .arg(&raw_cmd)
            .spawn()
            .map_err(|e| SaleemError::IoError(e.to_string()).to_user_payload())?;

        let _ = state.with_db(|conn| {
            record_audit_event(
                conn,
                "APP_UNINSTALLER_LAUNCHED",
                "SALEEM.Inventory",
                "LAUNCHED",
                &format!("app={}, source=Steam, uri={}", app_name, raw_cmd),
            )
        });
        return Ok(format!("Launched Steam uninstaller for {}", app_name));
    }

    if !raw_cmd.is_empty() {
        let (exe_part, args_part) = if let Some(stripped) = raw_cmd.strip_prefix('"') {
            if let Some(end_quote) = stripped.find('"') {
                let exe = &stripped[..end_quote];
                let rest = stripped[end_quote + 1..].trim();
                (exe.to_string(), rest.to_string())
            } else {
                (raw_cmd.clone(), String::new())
            }
        } else if let Some(exe_idx) = raw_cmd.to_ascii_lowercase().find(".exe") {
            let split_pos = exe_idx + 4;
            let exe = raw_cmd[..split_pos].trim();
            let rest = raw_cmd[split_pos..].trim();
            (exe.to_string(), rest.to_string())
        } else {
            let mut parts = raw_cmd.splitn(2, char::is_whitespace);
            let exe = parts.next().unwrap_or_default().to_string();
            let rest = parts.next().unwrap_or_default().trim().to_string();
            (exe, rest)
        };

        let mut cmd = std::process::Command::new(&exe_part);
        #[cfg(windows)]
        if !args_part.is_empty() {
            cmd.raw_arg(&args_part);
        }

        cmd.spawn()
            .map_err(|e| SaleemError::IoError(format!("Failed to launch uninstaller: {}", e)).to_user_payload())?;

        let _ = state.with_db(|conn| {
            record_audit_event(
                conn,
                "APP_UNINSTALLER_LAUNCHED",
                "SALEEM.Inventory",
                "LAUNCHED",
                &format!("app={}, source={}, exe={}", app_name, source, exe_part),
            )
        });
        return Ok(format!("Launched official uninstaller for {}", app_name));
    }

    // Fallback: Open Windows Installed Apps Settings
    std::process::Command::new("explorer.exe")
        .arg("ms-settings:appsfeatures")
        .spawn()
        .map_err(|e| SaleemError::IoError(e.to_string()).to_user_payload())?;

    let _ = state.with_db(|conn| {
        record_audit_event(
            conn,
            "APP_UNINSTALLER_LAUNCHED",
            "SALEEM.Inventory",
            "SETTINGS_OPENED",
            &format!("app={}, source={}", app_name, source),
        )
    });
    Ok(format!("Opened Windows Installed Apps settings for {}", app_name))
}

#[tauri::command]
pub fn open_in_explorer_command(path: String) -> Result<(), UserErrorPayload> {
    let normalized = normalize_lexical_windows_path(&path).map_err(|e| e.to_user_payload())?;
    let target = PathBuf::from(&normalized);
    if !target.exists() {
        return Err(SaleemError::PathNotFound(normalized).to_user_payload());
    }

    #[cfg(windows)]
    {
        if target.is_file() {
            std::process::Command::new("explorer.exe")
                .arg(format!("/select,{}", normalized))
                .spawn()
                .map_err(|e| SaleemError::IoError(e.to_string()).to_user_payload())?;
        } else {
            std::process::Command::new("explorer.exe")
                .arg(&normalized)
                .spawn()
                .map_err(|e| SaleemError::IoError(e.to_string()).to_user_payload())?;
        }
    }
    Ok(())
}

#[tauri::command]
pub fn get_audit_history_command(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<AuditEventRecord>, UserErrorPayload> {
    let max_items = limit.unwrap_or(50).min(500);
    state
        .with_db(|conn| list_recent_audit_events(conn, max_items))
        .map_err(|e| e.to_user_payload())
}

#[tauri::command]
pub fn update_ui_preferences_command(
    state: State<'_, AppState>,
    language: String,
    theme: String,
) -> Result<(), UserErrorPayload> {
    if !matches!(language.as_str(), "ar" | "en") {
        return Err(
            SaleemError::InvalidInput("Unsupported language code".into()).to_user_payload(),
        );
    }
    if !matches!(theme.as_str(), "dark" | "light") {
        return Err(SaleemError::InvalidInput("Unsupported theme mode".into()).to_user_payload());
    }

    state
        .with_db(|conn| {
            set_setting(conn, "ui.language", &language)?;
            set_setting(conn, "ui.theme", &theme)?;
            record_audit_event(
                conn,
                "PREFERENCES_UPDATED",
                "UI.Settings",
                "SUCCESS",
                &format!("language={}, theme={}", language, theme),
            )?;
            Ok(())
        })
        .map_err(|e| e.to_user_payload())
}

pub const DEFAULT_SUPABASE_PROJECT_URL: &str = match option_env!("SALEEM_SUPABASE_PROJECT_URL") {
    Some(val) => val,
    None => "https://YOUR_PROJECT_REF.supabase.co",
};
pub const DEFAULT_SUPABASE_POOLER_URI: &str = match option_env!("SALEEM_SUPABASE_POOLER_URI") {
    Some(val) => val,
    None => "host=YOUR_REGION.pooler.supabase.com port=5432 user=saleem_readonly.YOUR_PROJECT_REF password=YOUR_READONLY_PASSWORD dbname=postgres connect_timeout=10",
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudUpdateConfig {
    pub supabase_url: String,
    pub anon_key_configured: bool,
    pub anon_key_masked: String,
    pub current_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudReleaseRecord {
    pub version: String,
    pub channel: String,
    pub download_url: String,
    pub sha256_checksum: String,
    pub file_size_bytes: u64,
    pub release_notes_ar: String,
    pub release_notes_en: String,
    pub is_mandatory: bool,
    pub published_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudUpdateCheckResult {
    pub update_available: bool,
    pub current_version: String,
    pub latest_release: Option<CloudReleaseRecord>,
    pub checked_at: String,
}

fn is_newer_semver(current: &str, remote: &str) -> bool {
    let parse_parts = |v: &str| -> Vec<u64> {
        v.trim()
            .trim_start_matches('v')
            .split('.')
            .map(|p| p.parse::<u64>().unwrap_or(0))
            .collect()
    };
    let c = parse_parts(current);
    let r = parse_parts(remote);
    let len = c.len().max(r.len());
    for i in 0..len {
        let cv = *c.get(i).unwrap_or(&0);
        let rv = *r.get(i).unwrap_or(&0);
        if rv > cv {
            return true;
        }
        if rv < cv {
            return false;
        }
    }
    false
}

fn fetch_latest_release_via_pooler() -> Result<Option<CloudReleaseRecord>, SaleemError> {
    let connector = native_tls::TlsConnector::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .map_err(|e| SaleemError::IoError(format!("TLS initialization error: {}", e)))?;
    let tls = postgres_native_tls::MakeTlsConnector::new(connector);

    let mut client = postgres::Client::connect(DEFAULT_SUPABASE_POOLER_URI, tls)
        .map_err(|e| SaleemError::IoError(format!("Update server connection error: {}", e)))?;

    let rows = client
        .query(
            "SELECT row_to_json(r)::text AS j FROM (
                SELECT version, channel, download_url, sha256_checksum, file_size_bytes,
                       release_notes_ar, release_notes_en, is_mandatory, published_at
                FROM public.app_releases
                WHERE is_active = true
                ORDER BY published_at DESC
                LIMIT 1
            ) r;",
            &[],
        )
        .map_err(|e| SaleemError::IoError(format!("Update server query error: {}", e)))?;

    if let Some(row) = rows.first() {
        let json_str: String = row.get(0);
        let item: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|e| SaleemError::IoError(format!("Invalid release JSON: {}", e)))?;
        Ok(Some(CloudReleaseRecord {
            version: item
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("0.1.0")
                .to_string(),
            channel: item
                .get("channel")
                .and_then(|v| v.as_str())
                .unwrap_or("stable")
                .to_string(),
            download_url: item
                .get("download_url")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            sha256_checksum: item
                .get("sha256_checksum")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            file_size_bytes: item
                .get("file_size_bytes")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            release_notes_ar: item
                .get("release_notes_ar")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            release_notes_en: item
                .get("release_notes_en")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            is_mandatory: item
                .get("is_mandatory")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            published_at: item
                .get("published_at")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
        }))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub fn get_cloud_update_config_command(
    _state: State<'_, AppState>,
) -> Result<CloudUpdateConfig, UserErrorPayload> {
    Ok(CloudUpdateConfig {
        supabase_url: String::new(),
        anon_key_configured: true,
        anon_key_masked: "Official Stable Channel".to_string(),
        current_version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

#[tauri::command]
pub fn save_cloud_update_config_command(
    state: State<'_, AppState>,
    supabase_url: String,
    supabase_anon_key: String,
) -> Result<CloudUpdateConfig, UserErrorPayload> {
    let clean_url = supabase_url
        .trim()
        .trim_end_matches('/')
        .to_string();

    if !clean_url.starts_with("https://") {
        return Err(SaleemError::InvalidInput(
            "Supabase Project URL must start with https:// (e.g. https://your-project-ref.supabase.co)"
                .to_string(),
        )
        .to_user_payload());
    }

    let clean_key = supabase_anon_key.trim().to_string();
    if clean_key.starts_with("postgresql://") || clean_key.contains("@db.") {
        return Err(SaleemError::InvalidInput(
            "Do not use the PostgreSQL admin URI here. Please paste the public 'anon' API key (starts with eyJ... or sb_publishable_...) from Supabase -> Project Settings -> API."
                .to_string(),
        )
        .to_user_payload());
    }

    state
        .with_db(|conn| {
            set_setting(conn, "cloud.supabase_url", &clean_url)?;
            if !clean_key.is_empty() {
                set_setting(conn, "cloud.supabase_anon_key", &clean_key)?;
            }
            record_audit_event(
                conn,
                "CLOUD_UPDATER_CONFIGURED",
                "SALEEM.Updater",
                "SUCCESS",
                &format!("supabase_url={}", clean_url),
            )?;
            Ok(())
        })
        .map_err(|e| e.to_user_payload())?;

    get_cloud_update_config_command(state)
}

#[tauri::command]
pub fn check_for_cloud_updates_command(
    state: State<'_, AppState>,
) -> Result<CloudUpdateCheckResult, UserErrorPayload> {
    let (supabase_url, anon_key) = state
        .with_db(|conn| {
            let u = get_setting(conn, "cloud.supabase_url")?
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| DEFAULT_SUPABASE_PROJECT_URL.to_string());
            let k = get_setting(conn, "cloud.supabase_anon_key")?.unwrap_or_default();
            Ok((u, k))
        })
        .map_err(|e| e.to_user_payload())?;

    let current_version = env!("CARGO_PKG_VERSION").to_string();
    let checked_at = chrono::Utc::now().to_rfc3339();

    // If user hasn't pasted a custom REST anon key, connect directly via the built-in read-only Supabase pooler role!
    let latest_release = if anon_key.trim().is_empty() {
        fetch_latest_release_via_pooler().map_err(|e| e.to_user_payload())?
    } else {
        let endpoint = format!(
            "{}/rest/v1/app_releases?is_active=eq.true&order=published_at.desc&limit=1",
            supabase_url.trim_end_matches('/')
        );

        let output = std::process::Command::new(r"C:\Windows\System32\curl.exe")
            .arg("-sS")
            .arg("--max-time")
            .arg("10")
            .arg("-H")
            .arg(format!("apikey: {}", anon_key.trim()))
            .arg("-H")
            .arg(format!("Authorization: Bearer {}", anon_key.trim()))
            .arg("-H")
            .arg("Accept: application/json")
            .arg(&endpoint)
            .output()
            .map_err(|e| {
                SaleemError::IoError(format!("Failed to execute HTTPS request to Supabase: {}", e))
                    .to_user_payload()
            })?;

        let body = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !output.status.success() {
            return Err(SaleemError::IoError(format!(
                "Supabase API connection error: {}",
                body
            ))
            .to_user_payload());
        }

        let parsed_val: serde_json::Value = serde_json::from_str(&body).map_err(|e| {
            SaleemError::IoError(format!("Invalid JSON response from Supabase: {} ({})", e, body))
                .to_user_payload()
        })?;

        if let Some(err_msg) = parsed_val.get("message").and_then(|v| v.as_str()) {
            return Err(SaleemError::InvalidInput(format!(
                "Supabase API message: {}",
                err_msg
            ))
            .to_user_payload());
        }

        let arr = parsed_val.as_array().cloned().unwrap_or_default();
        arr.first().map(|item| CloudReleaseRecord {
            version: item
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("0.1.0")
                .to_string(),
            channel: item
                .get("channel")
                .and_then(|v| v.as_str())
                .unwrap_or("stable")
                .to_string(),
            download_url: item
                .get("download_url")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            sha256_checksum: item
                .get("sha256_checksum")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            file_size_bytes: item
                .get("file_size_bytes")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            release_notes_ar: item
                .get("release_notes_ar")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            release_notes_en: item
                .get("release_notes_en")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            is_mandatory: item
                .get("is_mandatory")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            published_at: item
                .get("published_at")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
        })
    };

    let update_available = latest_release
        .as_ref()
        .map(|rel| is_newer_semver(&current_version, &rel.version))
        .unwrap_or(false);

    let _ = state.with_db(|conn| {
        record_audit_event(
            conn,
            "CLOUD_UPDATE_CHECKED",
            "SALEEM.Updater",
            if update_available {
                "UPDATE_AVAILABLE"
            } else {
                "UP_TO_DATE"
            },
            &format!(
                "current={}, latest={}",
                current_version,
                latest_release
                    .as_ref()
                    .map(|r| r.version.as_str())
                    .unwrap_or("none")
            ),
        )
    });

    Ok(CloudUpdateCheckResult {
        update_available,
        current_version,
        latest_release,
        checked_at,
    })
}

#[tauri::command]
pub fn download_and_install_update_command(
    state: State<'_, AppState>,
    version: String,
    download_url: String,
    expected_sha256: String,
) -> Result<String, UserErrorPayload> {
    use sha2::{Digest, Sha256};

    let clean_ver: String = version
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '.' || *c == '-')
        .collect();
    let temp_installer_path =
        std::env::temp_dir().join(format!("SALEEM-Setup-v{}.exe", clean_ver));

    // 1. Download binary chunks in background directly from Supabase `public.app_release_binaries`
    let mut binary_bytes: Vec<u8> = Vec::new();
    if let Ok(connector) = native_tls::TlsConnector::builder()
        .danger_accept_invalid_certs(true)
        .build()
    {
        let tls = postgres_native_tls::MakeTlsConnector::new(connector);
        if let Ok(mut client) = postgres::Client::connect(DEFAULT_SUPABASE_POOLER_URI, tls) {
            if let Ok(rows) = client.query(
                "SELECT chunk_data FROM public.app_release_binaries WHERE version = $1 ORDER BY chunk_index ASC;",
                &[&version],
            ) {
                for row in rows {
                    let chunk: Vec<u8> = row.get(0);
                    binary_bytes.extend_from_slice(&chunk);
                }
            }
        }
    }

    // 2. Fallback to HTTPS download in background if binary was not hosted inside Supabase table
    if binary_bytes.is_empty() && download_url.trim().starts_with("https://") {
        let out = std::process::Command::new(r"C:\Windows\System32\curl.exe")
            .arg("-sSL")
            .arg("-f")
            .arg("--max-time")
            .arg("120")
            .arg("-o")
            .arg(&temp_installer_path)
            .arg(download_url.trim())
            .output()
            .map_err(|e| {
                SaleemError::IoError(format!("Background download error: {}", e)).to_user_payload()
            })?;
        if !out.status.success() {
            return Err(SaleemError::IoError(
                "تعذر تحميل ملف التحديث من الخادم (تأكد من رفع ملف التحديث عبر publish-update.ps1)."
                    .to_string(),
            )
            .to_user_payload());
        }
        binary_bytes = std::fs::read(&temp_installer_path)
            .map_err(|e| SaleemError::IoError(e.to_string()).to_user_payload())?;
    } else if !binary_bytes.is_empty() {
        std::fs::write(&temp_installer_path, &binary_bytes)
            .map_err(|e| SaleemError::IoError(e.to_string()).to_user_payload())?;
    } else {
        return Err(SaleemError::IoError(
            "لم يتم العثور على حزمة التحديث في الخادم السحابي.".to_string(),
        )
        .to_user_payload());
    }

    // 3. Verify SHA-256 Checksum
    let trimmed_expected = expected_sha256.trim().to_ascii_lowercase();
    if !trimmed_expected.is_empty() && !trimmed_expected.chars().all(|c| c == '0') {
        let mut hasher = Sha256::new();
        hasher.update(&binary_bytes);
        let actual_hex = hex::encode(hasher.finalize());
        if actual_hex != trimmed_expected {
            let _ = std::fs::remove_file(&temp_installer_path);
            return Err(SaleemError::InvalidInput(format!(
                "SHA-256 verification failed: expected {}, got {}",
                trimmed_expected, actual_hex
            ))
            .to_user_payload());
        }
    }

    // 4. Self-update running binary in-place (if a direct .exe or .zip was fetched, or via NSIS /S)
    let _ = state.with_db(|conn| {
        record_audit_event(
            conn,
            "CLOUD_UPDATE_INSTALLED",
            "SALEEM.Updater",
            "BACKGROUND_INSTALL_STARTED",
            &format!("version={}, bytes={}", version, binary_bytes.len()),
        )
    });

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        // Also check if direct self-update binary (version + "-exe") is available in Supabase for instant in-place replacement
        if let Ok(connector) = native_tls::TlsConnector::builder()
            .danger_accept_invalid_certs(true)
            .build()
        {
            let tls = postgres_native_tls::MakeTlsConnector::new(connector);
            if let Ok(mut client) = postgres::Client::connect(DEFAULT_SUPABASE_POOLER_URI, tls) {
                let exe_key = format!("{}-exe", version);
                if let Ok(rows) = client.query(
                    "SELECT chunk_data FROM public.app_release_binaries WHERE version = $1 ORDER BY chunk_index ASC;",
                    &[&exe_key],
                ) {
                    if !rows.is_empty() {
                        let mut zip_bytes: Vec<u8> = Vec::new();
                        for r in rows {
                            let c: Vec<u8> = r.get(0);
                            zip_bytes.extend_from_slice(&c);
                        }
                        let zip_path = std::env::temp_dir().join(format!("SALEEM-bin-{}.zip", clean_ver));
                        let extract_dir = std::env::temp_dir().join(format!("SALEEM-ext-{}", clean_ver));
                        let _ = std::fs::create_dir_all(&extract_dir);
                        if std::fs::write(&zip_path, &zip_bytes).is_ok() {
                            let _ = std::process::Command::new(r"C:\Windows\System32\tar.exe")
                                .arg("-xf")
                                .arg(&zip_path)
                                .arg("-C")
                                .arg(&extract_dir)
                                .creation_flags(CREATE_NO_WINDOW)
                                .output();
                            let extracted_exe = extract_dir.join("SALEEM.exe");
                            if extracted_exe.exists() {
                                if let Ok(current_exe_path) = std::env::current_exe() {
                                    let old_backup = current_exe_path.with_extension("exe.old");
                                    let _ = std::fs::remove_file(&old_backup);
                                    if std::fs::rename(&current_exe_path, &old_backup).is_ok() {
                                        if std::fs::copy(&extracted_exe, &current_exe_path).is_ok() {
                                            // Also update %LOCALAPPDATA%\SALEEM\SALEEM.exe if present
                                            if let Ok(local_app) = std::env::var("LOCALAPPDATA") {
                                                let installed = PathBuf::from(local_app).join("SALEEM").join("SALEEM.exe");
                                                if installed.exists() && installed != current_exe_path {
                                                    let _ = std::fs::copy(&extracted_exe, &installed);
                                                }
                                            }
                                            let _ = std::process::Command::new(&current_exe_path).spawn();
                                            std::process::exit(0);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let current_exe = std::env::current_exe()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default();
        let installer_str = temp_installer_path.to_string_lossy().into_owned();
        let script_path = std::env::temp_dir().join("saleem_bg_updater.ps1");
        let ps_script = format!(
            r#"$ErrorActionPreference = 'SilentlyContinue'
Start-Sleep -Milliseconds 600
Get-Process -Name 'SALEEM' | Stop-Process -Force
Start-Sleep -Milliseconds 500
Start-Process -FilePath '{installer}' -ArgumentList '/S' -Wait
$installedExe = Join-Path $env:LOCALAPPDATA 'SALEEM\SALEEM.exe'
if ((Test-Path $installedExe) -and ('{current}' -ne '') -and ('{current}' -ne $installedExe)) {{
    Copy-Item -Path $installedExe -Destination '{current}' -Force
}}
if (Test-Path $installedExe) {{
    Start-Process -FilePath $installedExe
}} elseif ('{current}' -ne '' -and (Test-Path '{current}')) {{
    Start-Process -FilePath '{current}'
}}
"#,
            installer = installer_str.replace('\'', "''"),
            current = current_exe.replace('\'', "''")
        );
        let _ = std::fs::write(&script_path, ps_script);

        std::process::Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-WindowStyle",
                "Hidden",
                "-ExecutionPolicy",
                "Bypass",
                "-File",
            ])
            .arg(&script_path)
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(|e| {
                SaleemError::IoError(format!("Failed to start background updater: {}", e))
                    .to_user_payload()
            })?;
    }

    Ok(temp_installer_path.to_string_lossy().into_owned())
}

// ============================================================================
// STARTUP MANAGER & SYSTEM BOOSTER (NEW FEATURE)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartupEntryItem {
    pub id: String,
    pub name: String,
    pub command_or_path: String,
    pub location_label: String,
    pub source_type: String, // "HKCU_RUN", "HKLM_RUN", "STARTUP_FOLDER_USER", "STARTUP_FOLDER_COMMON"
    pub can_remove_without_admin: bool,
    pub impact_estimate: String, // "High", "Medium", "Low"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMemoryTelemetry {
    pub memory_load_percent: u32,
    pub total_physical_bytes: u64,
    pub avail_physical_bytes: u64,
    pub used_physical_bytes: u64,
    pub total_pagefile_bytes: u64,
    pub avail_pagefile_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartupBoosterSnapshot {
    pub memory: SystemMemoryTelemetry,
    pub startup_items: Vec<StartupEntryItem>,
    pub thumbcache_reclaimable_bytes: u64,
    pub thumbcache_files_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoostActionOutcome {
    pub action_id: String,
    pub status: String,
    pub message_ar: String,
    pub message_en: String,
    pub bytes_freed: u64,
}

fn query_live_memory_telemetry() -> SystemMemoryTelemetry {
    #[cfg(windows)]
    unsafe {
        use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
        let mut mem: MEMORYSTATUSEX = std::mem::zeroed();
        mem.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
        if GlobalMemoryStatusEx(&mut mem) != 0 {
            let used = mem.ullTotalPhys.saturating_sub(mem.ullAvailPhys);
            return SystemMemoryTelemetry {
                memory_load_percent: mem.dwMemoryLoad,
                total_physical_bytes: mem.ullTotalPhys,
                avail_physical_bytes: mem.ullAvailPhys,
                used_physical_bytes: used,
                total_pagefile_bytes: mem.ullTotalPageFile,
                avail_pagefile_bytes: mem.ullAvailPageFile,
            };
        }
    }
    SystemMemoryTelemetry {
        memory_load_percent: 0,
        total_physical_bytes: 0,
        avail_physical_bytes: 0,
        used_physical_bytes: 0,
        total_pagefile_bytes: 0,
        avail_pagefile_bytes: 0,
    }
}

fn parse_reg_run_key(reg_path: &str, source_type: &str, location_label: &str, can_remove: bool) -> Vec<StartupEntryItem> {
    let mut items = Vec::new();
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        if let Ok(output) = std::process::Command::new(r"C:\Windows\System32\reg.exe")
            .args(["query", reg_path])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
        {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                for line in text.lines() {
                    let trimmed = line.trim();
                    if trimmed.is_empty() || trimmed.starts_with("HKEY_") {
                        continue;
                    }
                    let reg_marker = if trimmed.contains("    REG_SZ    ") {
                        Some("    REG_SZ    ")
                    } else if trimmed.contains("    REG_EXPAND_SZ    ") {
                        Some("    REG_EXPAND_SZ    ")
                    } else {
                        None
                    };
                    if let Some(marker) = reg_marker {
                        if let Some(idx) = trimmed.find(marker) {
                            let name = trimmed[..idx].trim().to_string();
                            let cmd = trimmed[idx + marker.len()..].trim().to_string();
                            if !name.is_empty() && !cmd.is_empty() {
                                let lower_cmd = cmd.to_ascii_lowercase();
                                let impact = if lower_cmd.contains("steam")
                                    || lower_cmd.contains("discord")
                                    || lower_cmd.contains("epic")
                                    || lower_cmd.contains("teams")
                                    || lower_cmd.contains("spotify")
                                    || lower_cmd.contains("adobe")
                                    || lower_cmd.contains("chrome")
                                    || lower_cmd.contains("edge")
                                {
                                    "High"
                                } else if lower_cmd.contains("update") || lower_cmd.contains("helper") {
                                    "Medium"
                                } else {
                                    "Low"
                                };
                                items.push(StartupEntryItem {
                                    id: format!("{}::{}", source_type, name),
                                    name,
                                    command_or_path: cmd,
                                    location_label: location_label.to_string(),
                                    source_type: source_type.to_string(),
                                    can_remove_without_admin: can_remove,
                                    impact_estimate: impact.to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    items
}

#[tauri::command]
pub fn get_startup_and_booster_status_command() -> Result<StartupBoosterSnapshot, UserErrorPayload> {
    // Clean up any leftover SALEEM.exe.old from self-update
    if let Ok(curr) = std::env::current_exe() {
        let old = curr.with_extension("exe.old");
        if old.exists() {
            let _ = std::fs::remove_file(old);
        }
    }

    let memory = query_live_memory_telemetry();
    let mut startup_items = Vec::new();

    startup_items.extend(parse_reg_run_key(
        r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
        "HKCU_RUN",
        "Registry (Current User Run)",
        true,
    ));
    startup_items.extend(parse_reg_run_key(
        r"HKLM\Software\Microsoft\Windows\CurrentVersion\Run",
        "HKLM_RUN",
        "Registry (Local Machine Run)",
        false,
    ));
    startup_items.extend(parse_reg_run_key(
        r"HKLM\Software\WOW6432Node\Software\Microsoft\Windows\CurrentVersion\Run",
        "HKLM_WOW64_RUN",
        "Registry (Local Machine x86 Run)",
        false,
    ));

    if let Ok(appdata) = std::env::var("APPDATA") {
        let user_startup = PathBuf::from(appdata).join(r"Microsoft\Windows\Start Menu\Programs\Startup");
        if let Ok(entries) = std::fs::read_dir(&user_startup) {
            for entry in entries.flatten() {
                let p = entry.path();
                let fname = entry.file_name().to_string_lossy().into_owned();
                if fname.eq_ignore_ascii_case("desktop.ini") {
                    continue;
                }
                startup_items.push(StartupEntryItem {
                    id: format!("STARTUP_FOLDER_USER::{}", p.to_string_lossy()),
                    name: fname,
                    command_or_path: p.to_string_lossy().into_owned(),
                    location_label: "User Startup Folder".to_string(),
                    source_type: "STARTUP_FOLDER_USER".to_string(),
                    can_remove_without_admin: true,
                    impact_estimate: "Medium".to_string(),
                });
            }
        }
    }

    let mut thumbcache_bytes = 0u64;
    let mut thumbcache_count = 0usize;
    if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
        let explorer_dir = PathBuf::from(local_appdata).join(r"Microsoft\Windows\Explorer");
        if let Ok(entries) = std::fs::read_dir(&explorer_dir) {
            for entry in entries.flatten() {
                let fname = entry.file_name().to_string_lossy().to_ascii_lowercase();
                if (fname.starts_with("thumbcache_") || fname.starts_with("iconcache_"))
                    && fname.ends_with(".db")
                {
                    if let Ok(meta) = entry.metadata() {
                        thumbcache_bytes += meta.len();
                        thumbcache_count += 1;
                    }
                }
            }
        }
    }

    Ok(StartupBoosterSnapshot {
        memory,
        startup_items,
        thumbcache_reclaimable_bytes: thumbcache_bytes,
        thumbcache_files_count: thumbcache_count,
    })
}

#[tauri::command]
pub fn remove_startup_item_command(
    state: State<'_, AppState>,
    name: String,
    command_or_path: String,
    source_type: String,
) -> Result<String, UserErrorPayload> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        if source_type == "HKCU_RUN" {
            let out = std::process::Command::new(r"C:\Windows\System32\reg.exe")
                .args([
                    "delete",
                    r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
                    "/v",
                    &name,
                    "/f",
                ])
                .creation_flags(CREATE_NO_WINDOW)
                .output()
                .map_err(|e| SaleemError::IoError(e.to_string()).to_user_payload())?;
            if !out.status.success() {
                return Err(SaleemError::IoError(
                    String::from_utf8_lossy(&out.stderr).trim().to_string(),
                )
                .to_user_payload());
            }
        } else if source_type == "STARTUP_FOLDER_USER" {
            let p = PathBuf::from(&command_or_path);
            if p.exists() {
                std::fs::remove_file(&p)
                    .map_err(|e| SaleemError::IoError(e.to_string()).to_user_payload())?;
            }
        } else {
            return Err(SaleemError::InvalidInput(
                "هذا العنصر مسجل على مستوى النظام (HKLM) ويتطلب صلاحيات مسؤول لتعديله.".into(),
            )
            .to_user_payload());
        }
    }

    let _ = state.with_db(|conn| {
        record_audit_event(
            conn,
            "STARTUP_ENTRY_REMOVED",
            "SALEEM.Booster",
            "REMOVED",
            &format!("name={}, source={}", name, source_type),
        )
    });

    Ok(name)
}

#[tauri::command]
pub fn execute_system_boost_action_command(
    state: State<'_, AppState>,
    action_id: String,
) -> Result<BoostActionOutcome, UserErrorPayload> {
    #[cfg(windows)]
    use std::os::windows::process::CommandExt;
    #[cfg(windows)]
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    match action_id.as_str() {
        "flush_dns" => {
            #[cfg(windows)]
            {
                let out = std::process::Command::new(r"C:\Windows\System32\ipconfig.exe")
                    .arg("/flushdns")
                    .creation_flags(CREATE_NO_WINDOW)
                    .output()
                    .map_err(|e| SaleemError::IoError(e.to_string()).to_user_payload())?;
                if !out.status.success() {
                    return Err(SaleemError::IoError("Failed to flush DNS cache".into()).to_user_payload());
                }
            }
            let _ = state.with_db(|conn| {
                record_audit_event(
                    conn,
                    "SYSTEM_BOOST_EXECUTED",
                    "SALEEM.Booster",
                    "SUCCESS",
                    "action=flush_dns",
                )
            });
            Ok(BoostActionOutcome {
                action_id,
                status: "SUCCESS".to_string(),
                message_ar: "تم تفريغ ذاكرة تخزين DNS المؤقتة (DNS Resolver Cache) وتنشيط استجابة الشبكة بنجاح!".to_string(),
                message_en: "Successfully flushed the Windows DNS Resolver Cache!".to_string(),
                bytes_freed: 0,
            })
        }
        "clean_thumbcache" => {
            let mut freed = 0u64;
            let mut deleted_count = 0usize;
            if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
                let explorer_dir = PathBuf::from(local_appdata).join(r"Microsoft\Windows\Explorer");
                if let Ok(entries) = std::fs::read_dir(&explorer_dir) {
                    for entry in entries.flatten() {
                        let fname = entry.file_name().to_string_lossy().to_ascii_lowercase();
                        if (fname.starts_with("thumbcache_") || fname.starts_with("iconcache_"))
                            && fname.ends_with(".db")
                        {
                            if let Ok(meta) = entry.metadata() {
                                let sz = meta.len();
                                if std::fs::remove_file(entry.path()).is_ok() {
                                    freed += sz;
                                    deleted_count += 1;
                                }
                            }
                        }
                    }
                }
            }
            let _ = state.with_db(|conn| {
                record_audit_event(
                    conn,
                    "SYSTEM_BOOST_EXECUTED",
                    "SALEEM.Booster",
                    "SUCCESS",
                    &format!("action=clean_thumbcache, files={}, bytes={}", deleted_count, freed),
                )
            });
            Ok(BoostActionOutcome {
                action_id,
                status: "SUCCESS".to_string(),
                message_ar: format!(
                    "تم تنظيف {} قاعدة بيانات مصغرات غير مقفلة وتحرير {:.2} MB بنجاح!",
                    deleted_count,
                    (freed as f64) / (1024.0 * 1024.0)
                ),
                message_en: format!(
                    "Cleaned {} unlocked thumbnail caches and freed {:.2} MB!",
                    deleted_count,
                    (freed as f64) / (1024.0 * 1024.0)
                ),
                bytes_freed: freed,
            })
        }
        _ => Err(SaleemError::InvalidInput("Unknown boost action".into()).to_user_payload()),
    }
}

#[tauri::command]
pub fn open_external_url_command(url: String) -> Result<(), UserErrorPayload> {
    let trimmed = url.trim();
    if !trimmed.starts_with("https://") {
        return Err(
            SaleemError::InvalidInput("Only HTTPS URLs are permitted.".into()).to_user_payload(),
        );
    }
    #[cfg(windows)]
    {
        std::process::Command::new("explorer.exe")
            .arg(trimmed)
            .spawn()
            .map_err(|e| SaleemError::IoError(e.to_string()).to_user_payload())?;
    }
    Ok(())
}

// ============================================================================
// STANDALONE MEMORY REDUCTION ENGINE (SALEEM.MemoryEngine.dll)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEngineStateResponse {
    pub config: saleem_memory_engine::MemoryEngineConfig,
    pub snapshot: saleem_memory_engine::MemorySnapshot,
    pub top_processes: Vec<saleem_memory_engine::ProcessMemoryEntry>,
    pub last_report: Option<saleem_memory_engine::MemoryCleanReport>,
    pub dll_status: NativeDllStatus,
}

fn format_bytes_human(bytes: u64) -> String {
    const GB: f64 = 1024.0 * 1024.0 * 1024.0;
    const MB: f64 = 1024.0 * 1024.0;
    if bytes as f64 >= GB {
        format!("{:.2} GB", bytes as f64 / GB)
    } else {
        format!("{:.0} MB", (bytes as f64 / MB).max(1.0))
    }
}

pub fn trigger_windows_toast_notification(title: &str, body: &str) {
    #[cfg(windows)]
    {
        let title_owned = title.replace('\'', " ").replace('"', " ");
        let body_owned = body.replace('\'', " ").replace('"', " ");
        std::thread::spawn(move || {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            let ps_script = format!(
                "$ErrorActionPreference = 'SilentlyContinue'; \
                [Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] > $null; \
                [Windows.Data.Xml.Dom.XmlDocument, Windows.Data.Xml.Dom.XmlDocument, ContentType = WindowsRuntime] > $null; \
                $template = '<toast><visual><binding template=\"ToastGeneric\"><text>{}</text><text>{}</text></binding></visual></toast>'; \
                $xml = New-Object Windows.Data.Xml.Dom.XmlDocument; \
                $xml.LoadXml($template); \
                $toast = [Windows.UI.Notifications.ToastNotification]::new($xml); \
                [Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier('Windows.SystemToast.SecurityAndMaintenance').Show($toast);",
                title_owned, body_owned
            );
            let _ = std::process::Command::new("powershell.exe")
                .args([
                    "-NoProfile",
                    "-NonInteractive",
                    "-WindowStyle",
                    "Hidden",
                    "-Command",
                    &ps_script,
                ])
                .creation_flags(CREATE_NO_WINDOW)
                .output();
        });
    }
}

fn check_app_in_hkcu_run() -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        if let Ok(out) = std::process::Command::new(r"C:\Windows\System32\reg.exe")
            .args([
                "query",
                r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
                "/v",
                "SALEEM",
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
        {
            return out.status.success();
        }
    }
    false
}

#[tauri::command]
pub fn get_memory_engine_status_command(
    state: State<'_, AppState>,
) -> Result<MemoryEngineStateResponse, UserErrorPayload> {
    let (
        clean_depth,
        auto_interval_minutes,
        auto_threshold_percent,
        clean_on_app_startup,
        run_in_background,
        show_notifications,
        last_report,
    ) = state
        .with_db(|conn| {
            let depth = get_setting(conn, "memory.clean_depth")?
                .unwrap_or_else(|| "Deep".to_string());
            let interval = get_setting(conn, "memory.auto_interval_minutes")?
                .and_then(|v| v.parse::<u32>().ok())
                .unwrap_or(0);
            let threshold = get_setting(conn, "memory.auto_threshold_percent")?
                .and_then(|v| v.parse::<u32>().ok())
                .unwrap_or(80);
            let on_start = get_setting(conn, "memory.clean_on_app_startup")?
                .map(|v| v == "true")
                .unwrap_or(false);
            let bg = get_setting(conn, "memory.run_in_background")?
                .map(|v| v == "true")
                .unwrap_or(true);
            let notify = get_setting(conn, "memory.show_notifications")?
                .map(|v| v == "true")
                .unwrap_or(true);
            let last_rep = get_setting(conn, "memory.last_report_json")?
                .and_then(|raw| serde_json::from_str::<saleem_memory_engine::MemoryCleanReport>(&raw).ok());
            Ok((depth, interval, threshold, on_start, bg, notify, last_rep))
        })
        .map_err(|e| e.to_user_payload())?;

    let launch_app_on_windows_startup = check_app_in_hkcu_run();
    let snapshot = saleem_memory_engine::get_memory_snapshot();
    let top_processes = saleem_memory_engine::get_top_memory_processes(12);
    let dll_status = inspect_engine_dll(
        "SALEEM Memory Engine (MemReduct)",
        "SALEEM.MemoryEngine.dll",
        "saleem_memory_engine.dll",
        b"memory_engine_abi_version\0",
        saleem_memory_engine::memory_engine_abi_version,
    );

    Ok(MemoryEngineStateResponse {
        config: saleem_memory_engine::MemoryEngineConfig {
            clean_depth,
            auto_interval_minutes,
            auto_threshold_percent,
            clean_on_app_startup,
            launch_app_on_windows_startup,
            run_in_background,
            show_notifications,
        },
        snapshot,
        top_processes,
        last_report,
        dll_status,
    })
}

#[tauri::command]
pub fn save_memory_engine_config_command(
    state: State<'_, AppState>,
    clean_depth: String,
    auto_interval_minutes: u32,
    auto_threshold_percent: u32,
    clean_on_app_startup: bool,
    launch_app_on_windows_startup: bool,
    run_in_background: bool,
    show_notifications: bool,
) -> Result<MemoryEngineStateResponse, UserErrorPayload> {
    let normalized_depth = saleem_memory_engine::MemoryCleanDepth::from_str_mode(&clean_depth)
        .as_str()
        .to_string();

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        if launch_app_on_windows_startup {
            if let Ok(curr_exe) = std::env::current_exe() {
                let exe_quoted = format!("\"{}\"", curr_exe.to_string_lossy());
                let _ = std::process::Command::new(r"C:\Windows\System32\reg.exe")
                    .args([
                        "add",
                        r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
                        "/v",
                        "SALEEM",
                        "/t",
                        "REG_SZ",
                        "/d",
                        &exe_quoted,
                        "/f",
                    ])
                    .creation_flags(CREATE_NO_WINDOW)
                    .output();
            }
        } else {
            let _ = std::process::Command::new(r"C:\Windows\System32\reg.exe")
                .args([
                    "delete",
                    r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
                    "/v",
                    "SALEEM",
                    "/f",
                ])
                .creation_flags(CREATE_NO_WINDOW)
                .output();
        }
    }

    state
        .with_db(|conn| {
            set_setting(conn, "memory.clean_depth", &normalized_depth)?;
            set_setting(
                conn,
                "memory.auto_interval_minutes",
                &auto_interval_minutes.to_string(),
            )?;
            set_setting(
                conn,
                "memory.auto_threshold_percent",
                &auto_threshold_percent.to_string(),
            )?;
            set_setting(
                conn,
                "memory.clean_on_app_startup",
                if clean_on_app_startup { "true" } else { "false" },
            )?;
            set_setting(
                conn,
                "memory.run_in_background",
                if run_in_background { "true" } else { "false" },
            )?;
            set_setting(
                conn,
                "memory.show_notifications",
                if show_notifications { "true" } else { "false" },
            )?;
            record_audit_event(
                conn,
                "MEMORY_ENGINE_CONFIGURED",
                "SALEEM.MemoryEngine",
                "SUCCESS",
                &format!(
                    "depth={}, interval={}m, threshold={}%, on_start={}, win_autostart={}, bg={}, notify={}",
                    normalized_depth,
                    auto_interval_minutes,
                    auto_threshold_percent,
                    clean_on_app_startup,
                    launch_app_on_windows_startup,
                    run_in_background,
                    show_notifications
                ),
            )?;
            Ok(())
        })
        .map_err(|e| e.to_user_payload())?;

    get_memory_engine_status_command(state)
}

#[tauri::command]
pub fn execute_memory_clean_command(
    state: State<'_, AppState>,
    depth: String,
    trigger_source: Option<String>,
) -> Result<saleem_memory_engine::MemoryCleanReport, UserErrorPayload> {
    let mode = saleem_memory_engine::MemoryCleanDepth::from_str_mode(&depth);
    let source = trigger_source.unwrap_or_else(|| "manual".to_string());
    let report = saleem_memory_engine::execute_memory_reduction_with_source(mode, &source)
        .map_err(|e| e.to_user_payload())?;

    let mut notify_enabled = true;
    let _ = state.with_db(|conn| {
        notify_enabled = get_setting(conn, "memory.show_notifications")?
            .map(|v| v == "true")
            .unwrap_or(true);
        if let Ok(json_str) = serde_json::to_string(&report) {
            let _ = set_setting(conn, "memory.last_report_json", &json_str);
        }
        record_audit_event(
            conn,
            "MEMORY_REDUCTION_COMPLETED",
            "SALEEM.MemoryEngine",
            "SUCCESS",
            &format!(
                "source={}, depth={}, freed_bytes={}, ws_freed={}, processes_trimmed={}/{}",
                source,
                report.depth,
                report.bytes_freed,
                report.working_set_freed_bytes,
                report.processes_trimmed,
                report.processes_scanned
            ),
        )
    });

    if notify_enabled {
        let freed_str = format_bytes_human(report.bytes_freed);
        let avail_str = format_bytes_human(report.after.avail_physical_bytes);
        let title = "⚡ SALEEM Memory Engine — تم تفريغ الرامات بنجاح";
        let body = format!(
            "تم تحرير {} من الرام ({} تطبيق) | الاستهلاك الآن {}% (المتوفر: {})",
            freed_str,
            report.processes_trimmed,
            report.after.memory_load_percent,
            avail_str
        );
        trigger_windows_toast_notification(title, &body);
    }

    Ok(report)
}

#[tauri::command]
pub fn trim_single_process_command(
    state: State<'_, AppState>,
    pid: u32,
    process_name: String,
) -> Result<u64, UserErrorPayload> {
    let freed =
        saleem_memory_engine::trim_single_process(pid).map_err(|e| e.to_user_payload())?;
    let _ = state.with_db(|conn| {
        record_audit_event(
            conn,
            "PROCESS_WORKING_SET_TRIMMED",
            &process_name,
            "SUCCESS",
            &format!("pid={}, freed_bytes={}", pid, freed),
        )
    });
    Ok(freed)
}

pub fn should_keep_running_in_background(app_handle: &tauri::AppHandle) -> bool {
    use tauri::Manager;
    if let Some(state) = app_handle.try_state::<AppState>() {
        if let Ok(bg) = state.with_db(|conn| {
            Ok(get_setting(conn, "memory.run_in_background")?
                .map(|v| v == "true")
                .unwrap_or(true))
        }) {
            return bg;
        }
    }
    false
}

pub fn spawn_background_memory_daemon(app_handle: tauri::AppHandle) {
    use std::time::{Duration, Instant};
    use tauri::Manager;

    std::thread::spawn(move || {
        let mut last_interval_clean = Instant::now();
        let mut last_threshold_clean: Option<Instant> = None;

        loop {
            std::thread::sleep(Duration::from_secs(15));

            let Some(state) = app_handle.try_state::<AppState>() else {
                continue;
            };

            let cfg_res = state.with_db(|conn| {
                let depth = get_setting(conn, "memory.clean_depth")?
                    .unwrap_or_else(|| "Deep".to_string());
                let interval = get_setting(conn, "memory.auto_interval_minutes")?
                    .and_then(|v| v.parse::<u32>().ok())
                    .unwrap_or(0);
                let threshold = get_setting(conn, "memory.auto_threshold_percent")?
                    .and_then(|v| v.parse::<u32>().ok())
                    .unwrap_or(80);
                let bg = get_setting(conn, "memory.run_in_background")?
                    .map(|v| v == "true")
                    .unwrap_or(true);
                let notify = get_setting(conn, "memory.show_notifications")?
                    .map(|v| v == "true")
                    .unwrap_or(true);
                Ok((depth, interval, threshold, bg, notify))
            });

            let Ok((depth_str, interval_mins, threshold_pct, run_bg, show_notify)) = cfg_res else {
                continue;
            };

            if !run_bg && interval_mins == 0 && threshold_pct == 0 {
                continue;
            }

            let snap = saleem_memory_engine::get_memory_snapshot();
            let mut trigger_reason: Option<&'static str> = None;

            if interval_mins > 0
                && last_interval_clean.elapsed()
                    >= Duration::from_secs((interval_mins as u64) * 60)
            {
                trigger_reason = Some("auto_interval");
                last_interval_clean = Instant::now();
            } else if threshold_pct > 0 && snap.memory_load_percent >= threshold_pct {
                let cooldown_ok = match last_threshold_clean {
                    Some(prev) => prev.elapsed() >= Duration::from_secs(120),
                    None => true,
                };
                if cooldown_ok {
                    trigger_reason = Some("auto_threshold");
                    last_threshold_clean = Some(Instant::now());
                }
            }

            if let Some(reason) = trigger_reason {
                let mode = saleem_memory_engine::MemoryCleanDepth::from_str_mode(&depth_str);
                if let Ok(report) =
                    saleem_memory_engine::execute_memory_reduction_with_source(mode, reason)
                {
                    let _ = state.with_db(|conn| {
                        if let Ok(json_str) = serde_json::to_string(&report) {
                            let _ = set_setting(conn, "memory.last_report_json", &json_str);
                        }
                        record_audit_event(
                            conn,
                            "MEMORY_AUTO_CLEAN_BACKGROUND",
                            "SALEEM.MemoryEngine",
                            "SUCCESS",
                            &format!(
                                "reason={}, depth={}, freed_bytes={}, load={}->{}%",
                                reason,
                                report.depth,
                                report.bytes_freed,
                                report.before.memory_load_percent,
                                report.after.memory_load_percent
                            ),
                        )
                    });

                    if show_notify {
                        let freed_str = format_bytes_human(report.bytes_freed);
                        let avail_str = format_bytes_human(report.after.avail_physical_bytes);
                        let title = if reason == "auto_threshold" {
                            "🛡️ SALEEM — تفريغ تلقائي ذكي للرامات بالخلفية"
                        } else {
                            "⏱️ SALEEM — تفريغ مجدول للرامات بالخلفية"
                        };
                        let body = format!(
                            "تم تحرير {} تلقائياً | انخفض الاستهلاك من {}% إلى {}% (المتوفر: {})",
                            freed_str,
                            report.before.memory_load_percent,
                            report.after.memory_load_percent,
                            avail_str
                        );
                        trigger_windows_toast_notification(title, &body);
                    }
                }
            }
        }
    });
}
