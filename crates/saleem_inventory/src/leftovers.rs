use crate::apps::discover_installed_registry_apps;
use crate::games::discover_installed_games;
use crate::models::{
    InstalledAppEntry, InventoryScanRequest, InventoryScanResult, LeftoverCandidate,
    LeftoverRemovalOutcome, LeftoverRemovalReport, LeftoverRemovalRequest,
};
use chrono::{DateTime, Utc};
use saleem_core::error::{SaleemError, SaleemResult};
use saleem_core::jobs::CancellationToken;
use saleem_core::paths::{
    assert_safe_for_modification, is_protected_system_path, is_reparse_point,
};
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime};
use uuid::Uuid;
use windows_sys::Win32::Foundation::{CloseHandle, GENERIC_READ, INVALID_HANDLE_VALUE};
use windows_sys::Win32::Storage::FileSystem::{
    CreateFileW, GetCompressedFileSizeW, FILE_ATTRIBUTE_NORMAL, OPEN_EXISTING,
};

fn compute_allocated_bytes(path: &Path, logical_size: u64) -> u64 {
    if logical_size == 0 {
        return 0;
    }
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
    const CLUSTER: u64 = 4096;
    ((logical_size + CLUSTER - 1) / CLUSTER) * CLUSTER
}

/// Core Windows, hardware driver, browser, and developer folders that must never be flagged as orphaned leftovers.
const PROTECTED_APPDATA_FOLDERS: &[&str] = &[
    "microsoft",
    "windows",
    "packages",
    "connecteddevicesplatform",
    "d3dscache",
    "temp",
    "crashdumps",
    "publishers",
    "comms",
    "diagnostics",
    "placeholdertilelogofolder",
    "virtualstore",
    "application data",
    "history",
    "start menu",
    "templates",
    "themes",
    "identities",
    "peerdistrepub",
    "squirreltemp",
    "cef",
    "google",
    "mozilla",
    "bravesoftware",
    "nvidia",
    "nvidia corporation",
    "amd",
    "ati",
    "intel",
    "realtek",
    "dolby",
    "package cache",
    "ssh",
    "usoprivate",
    "usoshared",
    "cargo",
    "rustup",
    "npm",
    "npm-cache",
    "pip",
    "uv",
    "git",
    "code",
    "vscode",
    "saleem",
    "com.saleem.desktop",
    "steam",
    "valve",
    "epic",
    "epicgameslauncher",
    "unrealengine",
];

/// Checks whether a single file is currently held open by any process using `CreateFileW(dwShareMode = 0)`.
pub fn is_file_locked(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    let wide_path: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        let handle = CreateFileW(
            wide_path.as_ptr(),
            GENERIC_READ,
            0,
            std::ptr::null(),
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            std::ptr::null_mut(),
        );
        if handle == INVALID_HANDLE_VALUE {
            true
        } else {
            let _ = CloseHandle(handle);
            false
        }
    }
}

fn normalize_alnum(input: &str) -> String {
    input
        .chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(|c| c.to_lowercase())
        .collect()
}

fn extract_tokens(input: &str) -> Vec<String> {
    input
        .split(|c: char| !c.is_alphanumeric())
        .map(|s| s.to_lowercase())
        .filter(|s| {
            s.len() >= 3
                && !matches!(
                    s.as_str(),
                    "inc"
                        | "llc"
                        | "ltd"
                        | "corp"
                        | "corporation"
                        | "software"
                        | "games"
                        | "studio"
                        | "studios"
                        | "app"
                        | "application"
                        | "the"
                        | "for"
                        | "windows"
                        | "x64"
                        | "x86"
                )
        })
        .collect()
}

fn is_protected_vendor_folder(folder_name: &str) -> bool {
    let lower = folder_name.trim().to_ascii_lowercase();
    if lower.starts_with('.') || lower.starts_with("regid.") {
        return true;
    }
    PROTECTED_APPDATA_FOLDERS.iter().any(|&p| p == lower)
}

/// Checks if `folder_name` matches any installed application's display name, publisher, or install directory name.
pub fn matches_any_installed_app(folder_name: &str, installed_apps: &[InstalledAppEntry]) -> bool {
    let folder_norm = normalize_alnum(folder_name);
    if folder_norm.len() < 3 {
        return true;
    }
    let folder_tokens = extract_tokens(folder_name);

    for app in installed_apps {
        let app_norm = normalize_alnum(&app.display_name);
        let pub_norm = normalize_alnum(&app.publisher);

        if (!app_norm.is_empty()
            && (app_norm.contains(&folder_norm) || folder_norm.contains(&app_norm)))
            || (!pub_norm.is_empty()
                && pub_norm != "unknownpublisher"
                && (pub_norm.contains(&folder_norm) || folder_norm.contains(&pub_norm)))
        {
            return true;
        }

        if let Some(loc) = &app.install_location {
            if let Some(leaf) = Path::new(loc).file_name().and_then(|n| n.to_str()) {
                let leaf_norm = normalize_alnum(leaf);
                if !leaf_norm.is_empty()
                    && (leaf_norm == folder_norm
                        || leaf_norm.contains(&folder_norm)
                        || folder_norm.contains(&leaf_norm))
                {
                    return true;
                }
            }
        }

        let app_tokens = extract_tokens(&app.display_name);
        for ft in &folder_tokens {
            if ft.len() >= 4 && app_tokens.iter().any(|at| at == ft) {
                return true;
            }
        }
    }

    false
}

struct FolderInspectionStats {
    logical_bytes: u64,
    allocated_bytes: u64,
    file_count: usize,
    latest_modified: SystemTime,
    contains_saved_games: bool,
    has_cache_or_logs: bool,
    has_locked_files: bool,
}

fn inspect_candidate_folder(folder_path: &Path, max_files_cap: usize) -> FolderInspectionStats {
    let mut stats = FolderInspectionStats {
        logical_bytes: 0,
        allocated_bytes: 0,
        file_count: 0,
        latest_modified: folder_path
            .metadata()
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH),
        contains_saved_games: false,
        has_cache_or_logs: false,
        has_locked_files: false,
    };

    let folder_name_lower = folder_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if folder_name_lower.contains("save") || folder_name_lower.contains("profile") {
        stats.contains_saved_games = true;
    }

    let mut stack = vec![folder_path.to_path_buf()];
    while let Some(curr) = stack.pop() {
        if stats.file_count >= max_files_cap {
            break;
        }
        let entries = match std::fs::read_dir(&curr) {
            Ok(rd) => rd,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let meta = match std::fs::symlink_metadata(&path) {
                Ok(m) => m,
                Err(_) => continue,
            };
            if is_reparse_point(&path) {
                continue;
            }

            let name_lower = entry.file_name().to_string_lossy().to_ascii_lowercase();

            if meta.is_dir() {
                if matches!(
                    name_lower.as_str(),
                    "save"
                        | "saves"
                        | "saved"
                        | "savedgames"
                        | "savedata"
                        | "profiles"
                        | "userdata"
                ) {
                    stats.contains_saved_games = true;
                }
                if matches!(
                    name_lower.as_str(),
                    "cache"
                        | "caches"
                        | "code cache"
                        | "gpucache"
                        | "shadercache"
                        | "logs"
                        | "crashreports"
                        | "crashes"
                        | "temp"
                ) {
                    stats.has_cache_or_logs = true;
                }
                stack.push(path);
            } else if meta.is_file() {
                stats.file_count += 1;
                let logical = meta.len();
                let allocated = compute_allocated_bytes(&path, logical);
                stats.logical_bytes = stats.logical_bytes.saturating_add(logical);
                stats.allocated_bytes = stats.allocated_bytes.saturating_add(allocated);

                if let Ok(mod_time) = meta.modified() {
                    if mod_time > stats.latest_modified {
                        stats.latest_modified = mod_time;
                    }
                }

                let ext = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or_default()
                    .to_ascii_lowercase();
                if matches!(ext.as_str(), "sav" | "save" | "slot") {
                    stats.contains_saved_games = true;
                }
                if matches!(ext.as_str(), "log" | "dmp" | "tmp" | "old" | "bak")
                    || name_lower.starts_with("unins000")
                {
                    stats.has_cache_or_logs = true;
                }

                // Sample lock status on first 15 files per candidate folder for fast responsiveness
                if !stats.has_locked_files && stats.file_count <= 15 && is_file_locked(&path) {
                    stats.has_locked_files = true;
                }
            }
        }
    }

    stats
}

fn allowed_leftover_roots(custom_sandbox_root: Option<&str>) -> Vec<(String, PathBuf)> {
    if let Some(sandbox) = custom_sandbox_root {
        return vec![("Sandbox".to_string(), PathBuf::from(sandbox))];
    }

    let mut roots = Vec::new();
    if let Ok(appdata) = std::env::var("APPDATA") {
        roots.push(("AppData\\Roaming".to_string(), PathBuf::from(appdata)));
    }
    if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
        let local_path = PathBuf::from(&local_appdata);
        roots.push(("AppData\\Local".to_string(), local_path.clone()));
        if let Some(parent) = local_path.parent() {
            let locallow = parent.join("LocalLow");
            if locallow.is_dir() {
                roots.push(("AppData\\LocalLow".to_string(), locallow));
            }
        }
    }
    if let Ok(program_data) = std::env::var("PROGRAMDATA") {
        roots.push(("ProgramData".to_string(), PathBuf::from(program_data)));
    }
    roots
}

/// Runs a full Application, Game & Orphaned Leftover Inventory scan.
pub fn scan_inventory_and_leftovers(
    request: &InventoryScanRequest,
    cancel_token: &CancellationToken,
) -> SaleemResult<InventoryScanResult> {
    let start_instant = Instant::now();
    let started_at = Utc::now().to_rfc3339();
    let scan_id = Uuid::new_v4().to_string();

    let installed_apps: Vec<InstalledAppEntry> =
        if let Some(mock_apps) = &request.mock_installed_apps {
            mock_apps.clone()
        } else {
            let mut apps = discover_installed_registry_apps();
            let games = discover_installed_games();
            apps.extend(games);
            apps
        };

    if cancel_token.is_cancelled() {
        return Err(SaleemError::ScanCancelled);
    }

    let total_games_count = installed_apps
        .iter()
        .filter(|a| a.scope == "Game Library")
        .count();
    let total_apps_count = installed_apps.len().saturating_sub(total_games_count);

    let roots = allowed_leftover_roots(request.custom_sandbox_root.as_deref());
    let now_sys = SystemTime::now();
    let mut leftover_candidates: Vec<LeftoverCandidate> = Vec::new();

    for (scope_label, root_dir) in roots {
        if cancel_token.is_cancelled() {
            return Err(SaleemError::ScanCancelled);
        }
        if !root_dir.is_dir() {
            continue;
        }

        let entries = match std::fs::read_dir(&root_dir) {
            Ok(rd) => rd,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            if cancel_token.is_cancelled() {
                return Err(SaleemError::ScanCancelled);
            }

            let path = entry.path();
            let meta = match std::fs::symlink_metadata(&path) {
                Ok(m) => m,
                Err(_) => continue,
            };
            if !meta.is_dir() || is_reparse_point(&path) {
                continue;
            }

            let folder_name = entry.file_name().to_string_lossy().into_owned();
            if is_protected_vendor_folder(&folder_name) {
                continue;
            }

            let path_str = path.to_string_lossy().into_owned();
            if is_protected_system_path(&path_str) {
                continue;
            }

            if matches_any_installed_app(&folder_name, &installed_apps) {
                continue;
            }

            let stats = inspect_candidate_folder(&path, 600);
            if stats.file_count == 0 || stats.logical_bytes == 0 {
                continue;
            }

            let age_secs = now_sys
                .duration_since(stats.latest_modified)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let age_days = age_secs / 86_400;
            let last_modified_at: DateTime<Utc> = stats.latest_modified.into();

            let (confidence, reason_en, reason_ar) = if stats.has_locked_files {
                (
                    "Low".to_string(),
                    "Contains files actively held open by a running process; skipped by default."
                        .to_string(),
                    "يحتوي على ملفات مفتوحة حالياً بواسطة عملية نشطة؛ يُستثنى افتراضياً.".to_string(),
                )
            } else if stats.contains_saved_games {
                (
                    "Low".to_string(),
                    "Orphaned folder with no matching installed app, but contains Saved Games / User Profiles (protected by default)."
                        .to_string(),
                    "مجلد يتيم لا يرتبط بتطبيق مثبت، لكنه يحتوي على ملفات حفظ ألعاب أو إعدادات مستخدم (محمي افتراضياً)."
                        .to_string(),
                )
            } else if stats.has_cache_or_logs || age_days >= 14 {
                (
                    "High".to_string(),
                    format!(
                        "No matching installed application or game in Registry/Launchers; contains orphaned cache/logs or inactive for {} days.",
                        age_days
                    ),
                    format!(
                        "لا يوجد برنامج أو لعبة مثبتة مطابقة في السجل أو المشغلات؛ يحتوي على بقايا كاش/سجلات أو غير مستخدم منذ {} يوماً.",
                        age_days
                    ),
                )
            } else {
                (
                    "Medium".to_string(),
                    format!(
                        "No matching uninstaller in Registry/Launchers (modified {} days ago); review before removal.",
                        age_days
                    ),
                    format!(
                        "لا يرتبط ببرنامج مثبت في السجل (تم تعديله منذ {} يوماً)؛ يُنصح بمراجعته قبل الحذف.",
                        age_days
                    ),
                )
            };

            let recommended_select =
                confidence == "High" && !stats.contains_saved_games && !stats.has_locked_files;

            leftover_candidates.push(LeftoverCandidate {
                id: format!("leftover:{}:{}", scope_label, folder_name),
                folder_name,
                path: path_str,
                scope_label: scope_label.clone(),
                logical_size_bytes: stats.logical_bytes,
                allocated_size_bytes: stats.allocated_bytes,
                file_count: stats.file_count,
                last_modified_at: last_modified_at.to_rfc3339(),
                age_days,
                confidence,
                reason_en,
                reason_ar,
                contains_saved_games_or_user_data: stats.contains_saved_games,
                has_locked_files: stats.has_locked_files,
                recommended_select,
            });
        }
    }

    // Sort leftovers by confidence (High first) then size descending
    leftover_candidates.sort_by(|a, b| {
        let rank = |c: &str| match c {
            "High" => 0,
            "Medium" => 1,
            _ => 2,
        };
        rank(&a.confidence)
            .cmp(&rank(&b.confidence))
            .then_with(|| b.logical_size_bytes.cmp(&a.logical_size_bytes))
    });

    let total_leftovers_count = leftover_candidates.len();
    let high_confidence_leftovers_count = leftover_candidates
        .iter()
        .filter(|c| c.confidence == "High")
        .count();
    let total_leftover_bytes: u64 = leftover_candidates
        .iter()
        .map(|c| c.logical_size_bytes)
        .sum();
    let high_confidence_reclaimable_bytes: u64 = leftover_candidates
        .iter()
        .filter(|c| c.confidence == "High")
        .map(|c| c.logical_size_bytes)
        .sum();

    let completed_at = Utc::now().to_rfc3339();
    let duration_ms = start_instant.elapsed().as_millis() as u64;

    Ok(InventoryScanResult {
        scan_id,
        started_at,
        completed_at,
        duration_ms,
        installed_apps,
        leftover_candidates,
        total_apps_count,
        total_games_count,
        total_leftovers_count,
        high_confidence_leftovers_count,
        total_leftover_bytes,
        high_confidence_reclaimable_bytes,
    })
}

/// Validates and removes user-selected orphaned leftover folders with full path, scope, and lock verification.
pub fn remove_verified_leftovers(
    request: &LeftoverRemovalRequest,
) -> SaleemResult<LeftoverRemovalReport> {
    if !request.user_confirmed {
        return Err(SaleemError::InvalidInput(
            "Leftover removal requires explicit user confirmation (user_confirmed = true)."
                .to_string(),
        ));
    }

    let started_at = Utc::now().to_rfc3339();
    let job_id = Uuid::new_v4().to_string();
    let allowed_roots: Vec<PathBuf> =
        allowed_leftover_roots(request.custom_sandbox_root.as_deref())
            .into_iter()
            .map(|(_, p)| p)
            .collect();

    let mut succeeded_count: usize = 0;
    let mut skipped_or_failed_count: usize = 0;
    let mut bytes_reclaimed: u64 = 0;
    let mut outcomes: Vec<LeftoverRemovalOutcome> = Vec::with_capacity(request.items.len());

    for item in &request.items {
        if is_protected_system_path(&item.path) {
            skipped_or_failed_count += 1;
            outcomes.push(LeftoverRemovalOutcome {
                path: item.path.clone(),
                status: "REJECTED_PROTECTED_PATH".to_string(),
                reason: "Blocked by SALEEM Protected System Path Policy.".to_string(),
                bytes_reclaimed: 0,
                files_removed: 0,
            });
            continue;
        }

        let target_buf = match assert_safe_for_modification(&item.path) {
            Ok(p) => p,
            Err(e) => {
                skipped_or_failed_count += 1;
                outcomes.push(LeftoverRemovalOutcome {
                    path: item.path.clone(),
                    status: "INVALID_PATH".to_string(),
                    reason: e.to_string(),
                    bytes_reclaimed: 0,
                    files_removed: 0,
                });
                continue;
            }
        };

        let target = target_buf.as_path();

        // Ensure target is strictly a child of one of the allowed leftover roots (and never the root itself!)
        let is_inside_allowed_root = allowed_roots.iter().any(|root| {
            target.starts_with(root) && target != root && target.parent() == Some(root.as_path())
        });

        if !is_inside_allowed_root {
            skipped_or_failed_count += 1;
            outcomes.push(LeftoverRemovalOutcome {
                path: item.path.clone(),
                status: "OUT_OF_SCOPE".to_string(),
                reason: "Target folder is outside allowed AppData/ProgramData leftover scopes."
                    .to_string(),
                bytes_reclaimed: 0,
                files_removed: 0,
            });
            continue;
        }

        if !target.exists() {
            skipped_or_failed_count += 1;
            outcomes.push(LeftoverRemovalOutcome {
                path: item.path.clone(),
                status: "ALREADY_REMOVED".to_string(),
                reason: "Folder no longer exists on disk.".to_string(),
                bytes_reclaimed: 0,
                files_removed: 0,
            });
            continue;
        }

        let stats = inspect_candidate_folder(target, 5000);
        if stats.has_locked_files {
            skipped_or_failed_count += 1;
            outcomes.push(LeftoverRemovalOutcome {
                path: item.path.clone(),
                status: "SKIPPED_LOCKED".to_string(),
                reason: "Folder contains files actively locked by a running process.".to_string(),
                bytes_reclaimed: 0,
                files_removed: 0,
            });
            continue;
        }

        match std::fs::remove_dir_all(target) {
            Ok(()) => {
                if !target.exists() {
                    succeeded_count += 1;
                    bytes_reclaimed = bytes_reclaimed.saturating_add(stats.logical_bytes);
                    outcomes.push(LeftoverRemovalOutcome {
                        path: item.path.clone(),
                        status: "REMOVED_VERIFIED".to_string(),
                        reason: format!(
                            "Verified removal of leftover folder ({} files).",
                            stats.file_count
                        ),
                        bytes_reclaimed: stats.logical_bytes,
                        files_removed: stats.file_count,
                    });
                } else {
                    skipped_or_failed_count += 1;
                    outcomes.push(LeftoverRemovalOutcome {
                        path: item.path.clone(),
                        status: "VERIFY_FAILED".to_string(),
                        reason: "Directory still exists after remove_dir_all.".to_string(),
                        bytes_reclaimed: 0,
                        files_removed: 0,
                    });
                }
            }
            Err(e) => {
                skipped_or_failed_count += 1;
                outcomes.push(LeftoverRemovalOutcome {
                    path: item.path.clone(),
                    status: "DELETE_ERROR".to_string(),
                    reason: format!("OS removal error: {}", e),
                    bytes_reclaimed: 0,
                    files_removed: 0,
                });
            }
        }
    }

    let completed_at = Utc::now().to_rfc3339();
    Ok(LeftoverRemovalReport {
        job_id,
        scan_id: request.scan_id.clone(),
        started_at,
        completed_at,
        total_requested: request.items.len(),
        succeeded_count,
        skipped_or_failed_count,
        bytes_reclaimed,
        outcomes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::games::parse_steam_acf_manifest;
    use crate::models::LeftoverRemovalItemRequest;
    use tempfile::tempdir;

    #[test]
    fn test_steam_acf_manifest_parser_extracts_exact_game_metadata() {
        let sample_acf = r#"
"AppState"
{
    "appid"     "730"
    "Universe"  "1"
    "name"      "Counter-Strike 2"
    "StateFlags"    "4"
    "installdir"    "Counter-Strike Global Offensive"
    "SizeOnDisk"    "38542190592"
}
"#;
        let steamapps = Path::new(r"D:\SteamLibrary\steamapps");
        let parsed = parse_steam_acf_manifest(sample_acf, steamapps).expect("Should parse ACF");
        assert_eq!(parsed.id, "steam:730");
        assert_eq!(parsed.display_name, "Counter-Strike 2");
        assert_eq!(parsed.size_bytes, 38_542_190_592);
        assert!(!parsed.is_size_estimated);
        assert_eq!(parsed.source, "Steam");
    }

    #[test]
    fn test_leftover_detector_preserves_installed_apps_and_protects_saved_games() {
        let dir = tempdir().unwrap();
        let sandbox = dir.path();

        // 1. Folder belonging to an active installed app ("Blender Foundation") -> must NOT be flagged
        let active_app_dir = sandbox.join("Blender Foundation");
        std::fs::create_dir_all(&active_app_dir).unwrap();
        std::fs::write(active_app_dir.join("config.txt"), b"active config").unwrap();

        // 2. Orphaned leftover folder with cache/logs -> must be flagged as High confidence & recommended_select = true
        let orphan_cache_dir = sandbox.join("OldUninstalledStudio").join("Cache");
        std::fs::create_dir_all(&orphan_cache_dir).unwrap();
        std::fs::write(orphan_cache_dir.join("shader.tmp"), vec![b'X'; 4096]).unwrap();

        // 3. Orphaned game folder containing SavedGames -> must be flagged with contains_saved_games_or_user_data = true & recommended_select = false
        let orphan_save_dir = sandbox.join("RetroRPGGame").join("SavedGames");
        std::fs::create_dir_all(&orphan_save_dir).unwrap();
        std::fs::write(orphan_save_dir.join("slot1.sav"), vec![b'S'; 2048]).unwrap();

        let mock_apps = vec![InstalledAppEntry {
            id: "reg:test:blender".to_string(),
            display_name: "Blender 4.2".to_string(),
            publisher: "Blender Foundation".to_string(),
            display_version: "4.2.0".to_string(),
            install_date: None,
            install_location: Some(r"C:\Program Files\Blender Foundation\Blender 4.2".to_string()),
            uninstall_string: None,
            has_uninstaller: true,
            size_bytes: 500_000_000,
            is_size_estimated: true,
            scope: "System (x64)".to_string(),
            source: "Windows Registry".to_string(),
        }];

        let cancel = CancellationToken::new();
        let res = scan_inventory_and_leftovers(
            &InventoryScanRequest {
                custom_sandbox_root: Some(sandbox.to_string_lossy().into_owned()),
                mock_installed_apps: Some(mock_apps),
            },
            &cancel,
        )
        .expect("Leftover scan should succeed");

        // Blender Foundation must not be in leftovers
        assert!(!res
            .leftover_candidates
            .iter()
            .any(|c| c.folder_name == "Blender Foundation"));

        // OldUninstalledStudio must be High confidence and recommended_select = true
        let studio = res
            .leftover_candidates
            .iter()
            .find(|c| c.folder_name == "OldUninstalledStudio")
            .expect("Should detect OldUninstalledStudio");
        assert_eq!(studio.confidence, "High");
        assert!(studio.recommended_select);
        assert!(!studio.contains_saved_games_or_user_data);

        // RetroRPGGame must have contains_saved_games_or_user_data = true and recommended_select = false
        let rpg = res
            .leftover_candidates
            .iter()
            .find(|c| c.folder_name == "RetroRPGGame")
            .expect("Should detect RetroRPGGame");
        assert!(rpg.contains_saved_games_or_user_data);
        assert!(!rpg.recommended_select);
        assert_eq!(rpg.confidence, "Low");
    }

    #[test]
    fn test_verified_leftover_removal_deletes_in_scope_and_rejects_out_of_scope() {
        let dir = tempdir().unwrap();
        let sandbox = dir.path();

        let orphan_dir = sandbox.join("OrphanAppLeftover");
        std::fs::create_dir_all(&orphan_dir).unwrap();
        std::fs::write(orphan_dir.join("leftover.log"), vec![b'L'; 8192]).unwrap();

        let req = LeftoverRemovalRequest {
            scan_id: "scan-test-01".to_string(),
            items: vec![
                LeftoverRemovalItemRequest {
                    path: orphan_dir.to_string_lossy().into_owned(),
                    expected_file_count: 1,
                    expected_logical_bytes: 8192,
                    confidence: "High".to_string(),
                },
                LeftoverRemovalItemRequest {
                    path: r"C:\Windows\System32".to_string(),
                    expected_file_count: 1,
                    expected_logical_bytes: 1024,
                    confidence: "High".to_string(),
                },
            ],
            user_confirmed: true,
            custom_sandbox_root: Some(sandbox.to_string_lossy().into_owned()),
        };

        let report = remove_verified_leftovers(&req).expect("Removal should execute");
        assert_eq!(report.succeeded_count, 1);
        assert_eq!(report.skipped_or_failed_count, 1);
        assert_eq!(report.bytes_reclaimed, 8192);
        assert!(!orphan_dir.exists());
    }
}
