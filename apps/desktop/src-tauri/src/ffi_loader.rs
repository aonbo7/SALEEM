use saleem_core::paths::{contains_traversal_or_nul, normalize_lexical_windows_path};
use saleem_core::{SaleemError, SaleemResult, SALEEM_ABI_VERSION};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Describes the verification and loading status of a native engine DLL (`Native/SALEEM.*.dll`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeDllStatus {
    pub logical_name: String,
    pub dll_file_name: String,
    pub resolved_path: String,
    pub loaded_via: String,
    pub abi_version: u32,
    pub expected_abi_version: u32,
    pub is_compatible: bool,
    pub status: String,
}

/// Validates that a DLL path is strictly inside a trusted application base directory,
/// is an absolute `.dll` path, contains no `..` traversal, and does not reside in writable
/// user `Temp`, `Tmp`, or `Downloads` folders (Anti-DLL-Hijacking Policy, DEC-003).
pub fn validate_trusted_dll_path(
    trusted_app_dir: &Path,
    candidate_dll_path: &Path,
) -> SaleemResult<PathBuf> {
    let raw_candidate = candidate_dll_path.to_string_lossy();
    let raw_base = trusted_app_dir.to_string_lossy();

    if contains_traversal_or_nul(&raw_candidate) || contains_traversal_or_nul(&raw_base) {
        return Err(SaleemError::FfiError(format!(
            "Rejected DLL path containing traversal or NUL: {}",
            raw_candidate
        )));
    }

    if !candidate_dll_path.is_absolute() || !trusted_app_dir.is_absolute() {
        return Err(SaleemError::FfiError(format!(
            "DLL loader requires absolute paths; got candidate='{}', base='{}'",
            raw_candidate, raw_base
        )));
    }

    let norm_candidate = normalize_lexical_windows_path(&raw_candidate)?;
    let norm_base = normalize_lexical_windows_path(&raw_base)?;

    let lower_candidate = norm_candidate.to_ascii_lowercase();
    let lower_base = norm_base.to_ascii_lowercase();

    if !lower_candidate.ends_with(".dll") {
        return Err(SaleemError::FfiError(format!(
            "Rejected non-DLL extension: {}",
            norm_candidate
        )));
    }

    // Never load DLLs from user-writable Temp/Downloads folders even if misconfigured
    let untrusted_markers = [
        r"\appdata\local\temp\",
        r"\windows\temp\",
        r"\downloads\",
        r"\public\",
    ];
    for marker in untrusted_markers {
        if lower_candidate.contains(marker) {
            return Err(SaleemError::FfiError(format!(
                "Blocked loading DLL from untrusted directory ({}): {}",
                marker, norm_candidate
            )));
        }
    }

    let expected_prefix = format!("{}\\", lower_base.trim_end_matches('\\'));
    if !lower_candidate.starts_with(&expected_prefix) {
        return Err(SaleemError::FfiError(format!(
            "Blocked loading DLL outside trusted application directory '{}': '{}'",
            norm_base, norm_candidate
        )));
    }

    Ok(PathBuf::from(norm_candidate))
}

/// Checks whether a reported engine ABI version matches `SALEEM_ABI_VERSION`.
pub fn check_abi_compatibility(reported_version: u32, expected_version: u32) -> SaleemResult<()> {
    if reported_version != expected_version {
        return Err(SaleemError::FfiError(format!(
            "ABI version mismatch: engine reported v{}, host expects v{}",
            reported_version, expected_version
        )));
    }
    Ok(())
}

/// Attempts to dynamically load a native engine DLL from `<exe_dir>/Native/<dll_name>` (Release layout)
/// or `<exe_dir>/<cargo_dll_name>` (Development layout) and verify its `*_abi_version` export.
/// If the external DLL file is not present on disk (e.g. during `cargo test`), gracefully falls back
/// to the linked `rlib` implementation while reporting exact load mode.
pub fn inspect_engine_dll(
    logical_name: &str,
    release_dll_name: &str,
    dev_dll_name: &str,
    abi_symbol: &[u8],
    rlib_abi_version_fn: extern "C" fn() -> u32,
) -> NativeDllStatus {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()));

    if let Some(base_dir) = exe_dir {
        let candidates = [
            base_dir.join("Native").join(release_dll_name),
            base_dir.join(release_dll_name),
            base_dir.join(dev_dll_name),
        ];

        for candidate in candidates {
            if candidate.exists() {
                if let Ok(validated_path) = validate_trusted_dll_path(&base_dir, &candidate) {
                    if let Ok(ver) = probe_dll_abi_version(&validated_path, abi_symbol) {
                        let compatible = check_abi_compatibility(ver, SALEEM_ABI_VERSION).is_ok();
                        return NativeDllStatus {
                            logical_name: logical_name.to_string(),
                            dll_file_name: release_dll_name.to_string(),
                            resolved_path: validated_path.to_string_lossy().into_owned(),
                            loaded_via: "Dynamic C-ABI DLL (cdylib)".to_string(),
                            abi_version: ver,
                            expected_abi_version: SALEEM_ABI_VERSION,
                            is_compatible: compatible,
                            status: if compatible {
                                "ACTIVE_DLL".to_string()
                            } else {
                                "ABI_MISMATCH".to_string()
                            },
                        };
                    }
                }
            }
        }
    }

    // Fallback to linked rlib C-ABI entrypoint (guarantees app never crashes if optional DLL file is absent)
    let ver = rlib_abi_version_fn();
    let compatible = check_abi_compatibility(ver, SALEEM_ABI_VERSION).is_ok();
    NativeDllStatus {
        logical_name: logical_name.to_string(),
        dll_file_name: release_dll_name.to_string(),
        resolved_path: format!("Static/Integrated C-ABI ({})", dev_dll_name),
        loaded_via: "Integrated Workspace Crate (rlib + C-ABI)".to_string(),
        abi_version: ver,
        expected_abi_version: SALEEM_ABI_VERSION,
        is_compatible: compatible,
        status: if compatible {
            "READY".to_string()
        } else {
            "ABI_MISMATCH".to_string()
        },
    }
}

fn probe_dll_abi_version(dll_path: &Path, abi_symbol: &[u8]) -> SaleemResult<u32> {
    unsafe {
        let lib = libloading::Library::new(dll_path).map_err(|e| {
            SaleemError::FfiError(format!(
                "Failed to load DLL '{}': {}",
                dll_path.display(),
                e
            ))
        })?;
        let func: libloading::Symbol<unsafe extern "C" fn() -> u32> =
            lib.get(abi_symbol).map_err(|e| {
                SaleemError::FfiError(format!(
                    "Missing ABI version symbol in '{}': {}",
                    dll_path.display(),
                    e
                ))
            })?;
        Ok(func())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accepts_valid_native_dll_inside_app_dir() {
        let app_dir = Path::new(r"C:\Program Files\SALEEM");
        let dll_path = Path::new(r"C:\Program Files\SALEEM\Native\SALEEM.Sentinel.dll");
        let res = validate_trusted_dll_path(app_dir, dll_path);
        assert!(res.is_ok());
    }

    #[test]
    fn test_rejects_dll_outside_app_dir_or_with_traversal() {
        let app_dir = Path::new(r"C:\Program Files\SALEEM");
        let outside = Path::new(r"C:\Evil\SALEEM.Sentinel.dll");
        assert!(validate_trusted_dll_path(app_dir, outside).is_err());

        let traversal = Path::new(r"C:\Program Files\SALEEM\Native\..\..\Evil\SALEEM.Sentinel.dll");
        assert!(validate_trusted_dll_path(app_dir, traversal).is_err());

        let relative = Path::new(r"Native\SALEEM.Sentinel.dll");
        assert!(validate_trusted_dll_path(app_dir, relative).is_err());
    }

    #[test]
    fn test_rejects_dll_in_temp_or_downloads() {
        let temp_base = Path::new(r"C:\Users\TestUser\AppData\Local\Temp\SALEEM");
        let temp_dll =
            Path::new(r"C:\Users\TestUser\AppData\Local\Temp\SALEEM\Native\SALEEM.Sentinel.dll");
        assert!(validate_trusted_dll_path(temp_base, temp_dll).is_err());

        let dl_base = Path::new(r"C:\Users\TestUser\Downloads\SALEEM");
        let dl_dll = Path::new(r"C:\Users\TestUser\Downloads\SALEEM\Native\SALEEM.Sentinel.dll");
        assert!(validate_trusted_dll_path(dl_base, dl_dll).is_err());
    }

    #[test]
    fn test_abi_version_compatibility_check() {
        assert!(check_abi_compatibility(1, 1).is_ok());
        assert!(check_abi_compatibility(2, 1).is_err());
        assert!(check_abi_compatibility(0, 1).is_err());
    }

    #[test]
    fn test_dynamic_load_compiled_release_dlls_if_present() {
        let release_dir = Path::new(r"D:\sleem\release");
        let dlls = [
            (
                "SALEEM.DiskEngine.dll",
                b"disk_engine_abi_version\0" as &[u8],
            ),
            (
                "SALEEM.CleanupEngine.dll",
                b"cleanup_engine_abi_version\0" as &[u8],
            ),
            ("SALEEM.Sentinel.dll", b"sentinel_abi_version\0" as &[u8]),
        ];

        for (dll_name, symbol) in dlls {
            let candidate = release_dir.join("Native").join(dll_name);
            if candidate.exists() {
                let validated = validate_trusted_dll_path(release_dir, &candidate)
                    .expect("Release DLL path should be trusted");
                let ver = probe_dll_abi_version(&validated, symbol)
                    .expect("Should dynamically load DLL and read ABI version");
                assert_eq!(ver, SALEEM_ABI_VERSION);
            }
        }
    }
}
