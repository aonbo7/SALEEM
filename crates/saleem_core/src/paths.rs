use crate::error::{SaleemError, SaleemResult};
use std::path::{Component, Path, PathBuf};

/// Protected Windows system directories and critical boot/OS files that SALEEM must NEVER delete,
/// quarantine, or modify under any cleanup rule or user action.
const PROTECTED_PATH_SEGMENTS: &[&str] = &[
    r"\windows\winsxs",
    r"\windows\system32",
    r"\windows\syswow64",
    r"\windows\sysnative",
    r"\windows\servicing",
    r"\windows\softwaredistribution",
    r"\windows\systemapps",
    r"\windows\boot",
    r"\windows\fonts",
    r"\windows\inf",
    r"\system volume information",
    r"\recovery",
    r"\$extend",
];

const PROTECTED_CRITICAL_FILENAMES: &[&str] = &[
    "bootmgr",
    "bootnxt",
    "pagefile.sys",
    "hiberfil.sys",
    "swapfile.sys",
    "ntoskrnl.exe",
    "hal.dll",
    "ntdll.dll",
    "kernel32.dll",
    "ntuser.dat",
    "usrclass.dat",
    "sam",
    "security",
    "software",
    "system",
];

/// Checks whether a raw path string or `Path` contains parent-directory traversal (`..`) components
/// or NUL bytes.
pub fn contains_traversal_or_nul(raw_path: &str) -> bool {
    if raw_path.trim().is_empty() || raw_path.contains('\0') {
        return true;
    }
    let normalized = raw_path.replace('/', "\\");
    for part in normalized.split('\\') {
        if part == ".." {
            return true;
        }
    }
    Path::new(raw_path)
        .components()
        .any(|c| matches!(c, Component::ParentDir))
}

/// Normalizes a path string for consistent policy evaluation without requiring the path to exist on disk.
pub fn normalize_lexical_windows_path(raw_path: &str) -> SaleemResult<String> {
    if contains_traversal_or_nul(raw_path) {
        return Err(SaleemError::InvalidInput(format!(
            "Path contains invalid traversal or NUL characters: {}",
            raw_path
        )));
    }
    let mut cleaned = raw_path.trim().replace('/', "\\");
    // Strip extended-length prefix `\\?\` if present for uniform matching
    if let Some(stripped) = cleaned.strip_prefix(r"\\?\") {
        cleaned = stripped.to_string();
    }
    while cleaned.contains(r"\\") && !cleaned.starts_with(r"\\") {
        cleaned = cleaned.replace(r"\\", r"\");
    }
    if cleaned.len() > 3 && cleaned.ends_with('\\') {
        cleaned.pop();
    }
    Ok(cleaned)
}

/// Checks if a given path targets a drive root (e.g., `C:\` or `C:`) or the Windows root directory (`C:\Windows`).
pub fn is_drive_or_windows_root(normalized_lower: &str) -> bool {
    let trimmed = normalized_lower.trim_end_matches('\\');
    // Match `c:`
    if trimmed.len() == 2 && trimmed.as_bytes()[1] == b':' {
        return true;
    }
    // Match `c:\windows` or `c:\program files` or `c:\program files (x86)` or `c:\users` exact root
    if trimmed.len() > 2 && trimmed.as_bytes()[1] == b':' {
        let sub = &trimmed[2..];
        if matches!(
            sub,
            r"\windows"
                | r"\program files"
                | r"\program files (x86)"
                | r"\programdata"
                | r"\users"
                | r"\$recycle.bin"
        ) {
            return true;
        }
    }
    false
}

/// Evaluates whether a path is a protected Windows system path that must NEVER be deleted, cleaned, or quarantined.
pub fn is_protected_system_path(raw_path: &str) -> bool {
    let Ok(normalized) = normalize_lexical_windows_path(raw_path) else {
        // Treat malformed/traversal paths as unsafe
        return true;
    };
    let lower = normalized.to_ascii_lowercase();

    if is_drive_or_windows_path_blocked(&lower) {
        return true;
    }

    if let Some(file_name) = Path::new(&lower).file_name().and_then(|n| n.to_str()) {
        if PROTECTED_CRITICAL_FILENAMES.contains(&file_name) {
            return true;
        }
    }

    false
}

fn is_drive_or_windows_path_blocked(lower: &str) -> bool {
    if is_drive_or_windows_root(lower) {
        return true;
    }

    // Strip drive letter prefix e.g. `c:` if present
    let without_drive = if lower.len() >= 2 && lower.as_bytes()[1] == b':' {
        &lower[2..]
    } else {
        lower
    };

    for protected in PROTECTED_PATH_SEGMENTS {
        if without_drive == *protected || without_drive.starts_with(&format!("{}\\", protected)) {
            return true;
        }
    }

    // Also block DriverStore anywhere under Windows
    if without_drive.contains(r"\driverstore\") || without_drive.ends_with(r"\driverstore") {
        return true;
    }

    false
}

/// Validates that a path is safe for modification (cleanup, deletion, or quarantine).
/// Returns `Err(SaleemError::ProtectedSystemPath)` or `Err(SaleemError::InvalidInput)` if forbidden.
pub fn assert_safe_for_modification(raw_path: &str) -> SaleemResult<PathBuf> {
    let normalized = normalize_lexical_windows_path(raw_path)?;
    if is_protected_system_path(&normalized) {
        return Err(SaleemError::ProtectedSystemPath(normalized));
    }
    Ok(PathBuf::from(normalized))
}

/// Checks if an existing path on disk is a Windows Reparse Point (Symlink or Junction).
pub fn is_reparse_point(path: &Path) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0400;
        if let Ok(meta) = std::fs::symlink_metadata(path) {
            return (meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT) != 0;
        }
        false
    }
    #[cfg(not(windows))]
    {
        std::fs::symlink_metadata(path)
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rejects_traversal_and_nul() {
        assert!(contains_traversal_or_nul(
            r"C:\Users\TestUser\..\..\Windows\System32"
        ));
        assert!(contains_traversal_or_nul("C:/Temp/../Windows"));
        assert!(contains_traversal_or_nul("C:\\Temp\\bad\0file.txt"));
        assert!(!contains_traversal_or_nul(
            r"C:\Users\TestUser\AppData\Local\Temp\file.tmp"
        ));
    }

    #[test]
    fn test_blocks_protected_windows_paths() {
        let forbidden = [
            r"C:\",
            r"D:\",
            r"C:\Windows",
            r"C:\Windows\System32",
            r"C:\Windows\System32\kernel32.dll",
            r"C:\Windows\System32\DriverStore\FileRepository\nv.inf",
            r"C:\Windows\WinSxS",
            r"C:\Windows\WinSxS\Manifests\x86_microsoft.manifest",
            r"C:\Windows\SysWOW64\cmd.exe",
            r"C:\Windows\SoftwareDistribution\Download\item",
            r"C:\System Volume Information\restore_point",
            r"D:\System Volume Information\tracking.log",
            r"C:\pagefile.sys",
            r"C:\hiberfil.sys",
            r"C:\bootmgr",
        ];

        for path in forbidden {
            assert!(
                is_protected_system_path(path),
                "Expected path to be protected: {}",
                path
            );
            assert!(
                matches!(
                    assert_safe_for_modification(path),
                    Err(SaleemError::ProtectedSystemPath(_))
                ),
                "Expected ProtectedSystemPath error for: {}",
                path
            );
        }
    }

    #[test]
    fn test_allows_safe_user_temp_and_cache_paths() {
        let allowed = [
            r"C:\Users\TestUser\AppData\Local\Temp\saleem_test.tmp",
            r"C:\Users\TestUser\AppData\Local\Temp\cache_folder\data.bin",
            r"D:\Games\OldUninstalledGame\ShaderCache\shader.bin",
        ];

        for path in allowed {
            assert!(
                !is_protected_system_path(path),
                "Expected path to be allowed: {}",
                path
            );
            assert!(assert_safe_for_modification(path).is_ok());
        }
    }
}
