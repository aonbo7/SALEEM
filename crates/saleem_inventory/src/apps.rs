use crate::models::InstalledAppEntry;
use std::collections::HashSet;
use std::os::windows::ffi::OsStrExt;
use windows_sys::Win32::Foundation::ERROR_SUCCESS;
use windows_sys::Win32::System::Registry::{
    RegCloseKey, RegEnumKeyExW, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_CURRENT_USER,
    HKEY_LOCAL_MACHINE, KEY_READ, REG_DWORD, REG_EXPAND_SZ, REG_SZ,
};

fn to_wide_null(s: &str) -> Vec<u16> {
    std::ffi::OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

unsafe fn query_reg_string(hkey: HKEY, value_name: &str) -> Option<String> {
    let wide_name = to_wide_null(value_name);
    let mut data_type: u32 = 0;
    let mut data_len: u32 = 0;

    let status = RegQueryValueExW(
        hkey,
        wide_name.as_ptr(),
        std::ptr::null_mut(),
        &mut data_type,
        std::ptr::null_mut(),
        &mut data_len,
    );

    if status != ERROR_SUCCESS
        || data_len == 0
        || (data_type != REG_SZ && data_type != REG_EXPAND_SZ)
    {
        return None;
    }

    let word_count = (data_len as usize + 1) / 2;
    let mut buffer: Vec<u16> = vec![0u16; word_count];
    let status2 = RegQueryValueExW(
        hkey,
        wide_name.as_ptr(),
        std::ptr::null_mut(),
        &mut data_type,
        buffer.as_mut_ptr() as *mut u8,
        &mut data_len,
    );

    if status2 != ERROR_SUCCESS {
        return None;
    }

    let end = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
    let text = String::from_utf16_lossy(&buffer[..end]).trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

unsafe fn query_reg_dword(hkey: HKEY, value_name: &str) -> Option<u32> {
    let wide_name = to_wide_null(value_name);
    let mut data_type: u32 = 0;
    let mut value: u32 = 0;
    let mut data_len: u32 = std::mem::size_of::<u32>() as u32;

    let status = RegQueryValueExW(
        hkey,
        wide_name.as_ptr(),
        std::ptr::null_mut(),
        &mut data_type,
        &mut value as *mut u32 as *mut u8,
        &mut data_len,
    );

    if status == ERROR_SUCCESS && data_type == REG_DWORD {
        Some(value)
    } else {
        None
    }
}

fn format_registry_date(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.len() == 8 && trimmed.chars().all(|c| c.is_ascii_digit()) {
        Some(format!(
            "{}-{}-{}",
            &trimmed[0..4],
            &trimmed[4..6],
            &trimmed[6..8]
        ))
    } else if !trimmed.is_empty() {
        Some(trimmed.to_string())
    } else {
        None
    }
}

unsafe fn enumerate_uninstall_hive(
    root_hkey: HKEY,
    subkey_path: &str,
    scope_label: &str,
    seen_keys: &mut HashSet<String>,
    out: &mut Vec<InstalledAppEntry>,
) {
    let wide_subkey = to_wide_null(subkey_path);
    let mut hkey_uninstall: HKEY = std::ptr::null_mut();

    let open_status = RegOpenKeyExW(
        root_hkey,
        wide_subkey.as_ptr(),
        0,
        KEY_READ,
        &mut hkey_uninstall,
    );
    if open_status != ERROR_SUCCESS || hkey_uninstall.is_null() {
        return;
    }

    let mut index: u32 = 0;
    loop {
        let mut name_buf = [0u16; 512];
        let mut name_len: u32 = name_buf.len() as u32;

        let enum_status = RegEnumKeyExW(
            hkey_uninstall,
            index,
            name_buf.as_mut_ptr(),
            &mut name_len,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );

        if enum_status != ERROR_SUCCESS {
            break;
        }
        index += 1;

        let key_name = String::from_utf16_lossy(&name_buf[..name_len as usize]);
        let wide_child = to_wide_null(&key_name);
        let mut hkey_app: HKEY = std::ptr::null_mut();

        if RegOpenKeyExW(
            hkey_uninstall,
            wide_child.as_ptr(),
            0,
            KEY_READ,
            &mut hkey_app,
        ) != ERROR_SUCCESS
            || hkey_app.is_null()
        {
            continue;
        }

        // Skip Windows internal system components
        let sys_comp = query_reg_dword(hkey_app, "SystemComponent").unwrap_or(0);
        let parent_key = query_reg_string(hkey_app, "ParentKeyName");
        let release_type = query_reg_string(hkey_app, "ReleaseType").unwrap_or_default();

        if sys_comp == 1
            || parent_key.is_some()
            || release_type.eq_ignore_ascii_case("Security Update")
            || release_type.eq_ignore_ascii_case("Update")
            || release_type.eq_ignore_ascii_case("Hotfix")
        {
            let _ = RegCloseKey(hkey_app);
            continue;
        }

        if let Some(display_name) = query_reg_string(hkey_app, "DisplayName") {
            let publisher = query_reg_string(hkey_app, "Publisher")
                .unwrap_or_else(|| "Unknown Publisher".to_string());
            let display_version =
                query_reg_string(hkey_app, "DisplayVersion").unwrap_or_else(|| "—".to_string());
            let install_date =
                query_reg_string(hkey_app, "InstallDate").and_then(|d| format_registry_date(&d));
            let install_location = query_reg_string(hkey_app, "InstallLocation")
                .map(|p| p.trim_matches('"').trim_end_matches('\\').to_string())
                .filter(|p| !p.is_empty());
            let uninstall_string = query_reg_string(hkey_app, "UninstallString")
                .or_else(|| query_reg_string(hkey_app, "QuietUninstallString"));
            let has_uninstaller = uninstall_string.is_some();

            // Registry EstimatedSize is stored in KiB (1024 bytes)
            let estimated_kb = query_reg_dword(hkey_app, "EstimatedSize").unwrap_or(0) as u64;
            let size_bytes = estimated_kb.saturating_mul(1024);

            let dedup_token = format!(
                "{}|{}",
                display_name.to_ascii_lowercase(),
                display_version.to_ascii_lowercase()
            );

            if seen_keys.insert(dedup_token) {
                out.push(InstalledAppEntry {
                    id: format!("reg:{}:{}", scope_label, key_name),
                    display_name,
                    publisher,
                    display_version,
                    install_date,
                    install_location,
                    uninstall_string,
                    has_uninstaller,
                    size_bytes,
                    is_size_estimated: true,
                    scope: scope_label.to_string(),
                    source: "Windows Registry".to_string(),
                });
            }
        }

        let _ = RegCloseKey(hkey_app);
    }

    let _ = RegCloseKey(hkey_uninstall);
}

/// Discovers installed desktop applications from Windows Registry Uninstall hives (x64, WOW6432Node x86, and Current User).
pub fn discover_installed_registry_apps() -> Vec<InstalledAppEntry> {
    let mut apps = Vec::new();
    let mut seen_keys = HashSet::new();

    unsafe {
        enumerate_uninstall_hive(
            HKEY_LOCAL_MACHINE,
            "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
            "System (x64)",
            &mut seen_keys,
            &mut apps,
        );
        enumerate_uninstall_hive(
            HKEY_LOCAL_MACHINE,
            "SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
            "System (x86)",
            &mut seen_keys,
            &mut apps,
        );
        enumerate_uninstall_hive(
            HKEY_CURRENT_USER,
            "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
            "Current User",
            &mut seen_keys,
            &mut apps,
        );
    }

    apps.sort_by(|a, b| {
        a.display_name
            .to_ascii_lowercase()
            .cmp(&b.display_name.to_ascii_lowercase())
    });
    apps
}
