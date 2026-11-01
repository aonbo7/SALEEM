use crate::models::InstalledAppEntry;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Extracts a VDF/ACF quoted key-value pair from a single line, e.g. `"name" "Counter-Strike 2"`.
pub fn parse_vdf_key_value_line(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim();
    if !trimmed.starts_with('"') {
        return None;
    }
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = trimmed.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\\' && in_quotes {
            if let Some(&next_ch) = chars.peek() {
                current.push(next_ch);
                chars.next();
                continue;
            }
        }
        if ch == '"' {
            if in_quotes {
                parts.push(current.clone());
                current.clear();
                in_quotes = false;
                if parts.len() == 2 {
                    break;
                }
            } else {
                in_quotes = true;
            }
        } else if in_quotes {
            current.push(ch);
        }
    }

    if parts.len() == 2 {
        Some((parts[0].clone(), parts[1].clone()))
    } else {
        None
    }
}

/// Parses a Steam `appmanifest_*.acf` content string into an `InstalledAppEntry`.
pub fn parse_steam_acf_manifest(content: &str, steamapps_dir: &Path) -> Option<InstalledAppEntry> {
    let mut appid = String::new();
    let mut name = String::new();
    let mut installdir = String::new();
    let mut size_on_disk: u64 = 0;

    for line in content.lines() {
        if let Some((k, v)) = parse_vdf_key_value_line(line) {
            match k.to_ascii_lowercase().as_str() {
                "appid" => appid = v,
                "name" => name = v,
                "installdir" => installdir = v,
                "sizeondisk" => size_on_disk = v.parse::<u64>().unwrap_or(0),
                _ => {}
            }
        }
    }

    if appid.is_empty() || name.is_empty() {
        return None;
    }

    // Skip Steamworks Common Redistributables internal package
    if appid == "228980" || name.eq_ignore_ascii_case("Steamworks Common Redistributables") {
        return None;
    }

    let common_path = steamapps_dir.join("common").join(&installdir);
    let install_location = Some(common_path.to_string_lossy().into_owned());

    Some(InstalledAppEntry {
        id: format!("steam:{}", appid),
        display_name: name,
        publisher: "Steam Game Library".to_string(),
        display_version: format!("AppID {}", appid),
        install_date: None,
        install_location,
        uninstall_string: Some(format!("steam://uninstall/{}", appid)),
        has_uninstaller: true,
        size_bytes: size_on_disk,
        is_size_estimated: false,
        scope: "Game Library".to_string(),
        source: "Steam".to_string(),
    })
}

/// Discovers installed games across Steam and Epic Games manifests on the system.
pub fn discover_installed_games() -> Vec<InstalledAppEntry> {
    let mut games = Vec::new();
    let mut seen_ids = HashSet::new();

    // 1. Discover candidate Steam library roots
    let mut steam_roots: Vec<PathBuf> = vec![
        PathBuf::from(r"C:\Program Files (x86)\Steam"),
        PathBuf::from(r"C:\Program Files\Steam"),
        PathBuf::from(r"C:\SteamLibrary"),
        PathBuf::from(r"D:\SteamLibrary"),
        PathBuf::from(r"D:\Steam"),
        PathBuf::from(r"E:\SteamLibrary"),
        PathBuf::from(r"F:\SteamLibrary"),
    ];

    // Expand from libraryfolders.vdf if present
    let initial_roots = steam_roots.clone();
    for root in initial_roots {
        let vdf_path = root.join("steamapps").join("libraryfolders.vdf");
        if let Ok(content) = std::fs::read_to_string(&vdf_path) {
            for line in content.lines() {
                if let Some((k, v)) = parse_vdf_key_value_line(line) {
                    if k.eq_ignore_ascii_case("path") {
                        let candidate = PathBuf::from(v.replace("\\\\", "\\"));
                        if !steam_roots.contains(&candidate) {
                            steam_roots.push(candidate);
                        }
                    }
                }
            }
        }
    }

    for root in steam_roots {
        let steamapps = root.join("steamapps");
        if !steamapps.is_dir() {
            continue;
        }
        if let Ok(entries) = std::fs::read_dir(&steamapps) {
            for entry in entries.flatten() {
                let fname = entry.file_name().to_string_lossy().to_ascii_lowercase();
                if fname.starts_with("appmanifest_") && fname.ends_with(".acf") {
                    if let Ok(content) = std::fs::read_to_string(entry.path()) {
                        if let Some(game) = parse_steam_acf_manifest(&content, &steamapps) {
                            if seen_ids.insert(game.id.clone()) {
                                games.push(game);
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. Discover Epic Games Launcher manifests
    if let Ok(program_data) = std::env::var("PROGRAMDATA") {
        let epic_manifests = PathBuf::from(program_data)
            .join("Epic")
            .join("EpicGamesLauncher")
            .join("Data")
            .join("Manifests");
        if epic_manifests.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&epic_manifests) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path
                        .extension()
                        .and_then(|e| e.to_str())
                        .map(|e| e.eq_ignore_ascii_case("item"))
                        .unwrap_or(false)
                    {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                                let app_name = json
                                    .get("AppName")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or_default();
                                let display_name = json
                                    .get("DisplayName")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or_default();
                                let install_location = json
                                    .get("InstallLocation")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string());
                                let install_size = json
                                    .get("InstallSize")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0);
                                let version = json
                                    .get("AppVersionString")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("—")
                                    .to_string();

                                if !display_name.is_empty() && !app_name.is_empty() {
                                    let id = format!("epic:{}", app_name);
                                    if seen_ids.insert(id.clone()) {
                                        games.push(InstalledAppEntry {
                                            id,
                                            display_name: display_name.to_string(),
                                            publisher: "Epic Games".to_string(),
                                            display_version: version,
                                            install_date: None,
                                            install_location,
                                            uninstall_string: None,
                                            has_uninstaller: false,
                                            size_bytes: install_size,
                                            is_size_estimated: false,
                                            scope: "Game Library".to_string(),
                                            source: "Epic Games".to_string(),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    games
}
