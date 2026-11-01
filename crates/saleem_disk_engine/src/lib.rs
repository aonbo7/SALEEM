pub mod duplicates;
pub mod models;
pub mod scanner;

pub use duplicates::{find_duplicates_with_progress, verify_and_remove_duplicate_selections};
pub use models::{
    CategoryStat, DiskScanRequest, DiskScanResult, DuplicateFileMember, DuplicateGroup,
    DuplicateRemovalGroupSelection, DuplicateRemovalReport, DuplicateScanRequest,
    DuplicateScanResult, EmptyRequest, EngineHealthInfo, ExtensionStat, ScannedFileEntry,
    ScannedFolderSummary, VolumeInfo,
};
pub use scanner::{
    classify_extension_category, compute_allocated_size, scan_directory_with_progress,
};

use saleem_core::ffi::{
    ffi_execute_json, ffi_free_buffer_impl, SaleemFfiBuffer, SALEEM_ABI_VERSION,
};
use saleem_core::{CancellationToken, SaleemError, SaleemResult};

pub const ENGINE_NAME: &str = "SALEEM Disk Engine";
pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn get_engine_health() -> EngineHealthInfo {
    EngineHealthInfo {
        engine_name: ENGINE_NAME.to_string(),
        engine_version: ENGINE_VERSION.to_string(),
        abi_version: SALEEM_ABI_VERSION,
        status: "READY".to_string(),
    }
}

/// Enumerates all mounted logical volumes on Windows using official Win32 Storage APIs.
pub fn list_volumes() -> SaleemResult<Vec<VolumeInfo>> {
    #[cfg(windows)]
    {
        use windows_sys::Win32::Storage::FileSystem::{
            GetDiskFreeSpaceExW, GetDriveTypeW, GetLogicalDrives, GetVolumeInformationW,
        };

        const DRIVE_REMOVABLE: u32 = 2;
        const DRIVE_FIXED: u32 = 3;
        const DRIVE_REMOTE: u32 = 4;
        const DRIVE_CDROM: u32 = 5;
        const DRIVE_RAMDISK: u32 = 6;

        let mask = unsafe { GetLogicalDrives() };
        if mask == 0 {
            return Err(SaleemError::IoError(
                "GetLogicalDrives returned 0".to_string(),
            ));
        }

        let mut volumes = Vec::new();
        for bit in 0..26u32 {
            if (mask & (1 << bit)) == 0 {
                continue;
            }
            let letter = (b'A' + bit as u8) as char;
            let mount = format!("{}:\\", letter);
            let wide_root: Vec<u16> = mount.encode_utf16().chain(std::iter::once(0)).collect();

            let drive_type_code = unsafe { GetDriveTypeW(wide_root.as_ptr()) };
            let (drive_type_str, is_removable) = match drive_type_code {
                DRIVE_REMOVABLE => ("Removable", true),
                DRIVE_FIXED => ("Fixed Local Disk", false),
                DRIVE_REMOTE => ("Network Drive", false),
                DRIVE_CDROM => ("Optical / CD-ROM", true),
                DRIVE_RAMDISK => ("RAM Disk", false),
                _ => ("Unknown", false),
            };

            let mut label_buf = [0u16; 261];
            let mut fs_buf = [0u16; 261];
            let mut serial_num: u32 = 0;
            let mut max_comp_len: u32 = 0;
            let mut fs_flags: u32 = 0;

            let vol_ok = unsafe {
                GetVolumeInformationW(
                    wide_root.as_ptr(),
                    label_buf.as_mut_ptr(),
                    label_buf.len() as u32,
                    &mut serial_num,
                    &mut max_comp_len,
                    &mut fs_flags,
                    fs_buf.as_mut_ptr(),
                    fs_buf.len() as u32,
                )
            };

            let mut free_available: u64 = 0;
            let mut total_bytes: u64 = 0;
            let mut total_free_bytes: u64 = 0;

            let space_ok = unsafe {
                GetDiskFreeSpaceExW(
                    wide_root.as_ptr(),
                    &mut free_available,
                    &mut total_bytes,
                    &mut total_free_bytes,
                )
            };

            let is_ready = vol_ok != 0 && space_ok != 0;
            let volume_label = if vol_ok != 0 {
                wide_null_terminated_to_string(&label_buf)
            } else {
                String::new()
            };
            let file_system = if vol_ok != 0 {
                wide_null_terminated_to_string(&fs_buf)
            } else {
                "Unavailable".to_string()
            };

            let used_bytes = total_bytes.saturating_sub(total_free_bytes);
            let usage_percent = if total_bytes > 0 {
                ((used_bytes as f64 / total_bytes as f64) * 1000.0).round() / 10.0
            } else {
                0.0
            };

            volumes.push(VolumeInfo {
                mount_point: mount,
                volume_label,
                file_system,
                total_bytes,
                free_bytes: total_free_bytes,
                used_bytes,
                usage_percent,
                drive_type: drive_type_str.to_string(),
                is_removable,
                is_ready,
            });
        }

        Ok(volumes)
    }

    #[cfg(not(windows))]
    {
        Ok(vec![])
    }
}

#[cfg(windows)]
fn wide_null_terminated_to_string(buf: &[u16]) -> String {
    let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..end])
}

// ============================================================================
// C-ABI Exports for `SALEEM.DiskEngine.dll`
// ============================================================================

#[no_mangle]
pub extern "C" fn disk_engine_abi_version() -> u32 {
    SALEEM_ABI_VERSION
}

/// Frees a `SaleemFfiBuffer` allocated by `SALEEM.DiskEngine.dll`.
///
/// # Safety
/// Must only be called with a buffer returned by `SALEEM.DiskEngine.dll`.
#[no_mangle]
pub unsafe extern "C" fn disk_engine_free_buffer(buf: SaleemFfiBuffer) {
    ffi_free_buffer_impl(buf);
}

/// Returns engine health metadata over the C ABI.
///
/// # Safety
/// `input_ptr` must be valid for `input_len` bytes if `input_len > 0`.
#[no_mangle]
pub unsafe extern "C" fn disk_engine_health_ffi(
    input_ptr: *const u8,
    input_len: usize,
) -> SaleemFfiBuffer {
    ffi_execute_json(input_ptr, input_len, |_: EmptyRequest| {
        Ok(get_engine_health())
    })
}

/// Lists mounted Windows volumes over the C ABI.
///
/// # Safety
/// `input_ptr` must be valid for `input_len` bytes if `input_len > 0`.
#[no_mangle]
pub unsafe extern "C" fn disk_engine_list_volumes_ffi(
    input_ptr: *const u8,
    input_len: usize,
) -> SaleemFfiBuffer {
    ffi_execute_json(input_ptr, input_len, |_: EmptyRequest| list_volumes())
}

/// Executes a synchronous directory scan over the C ABI.
///
/// # Safety
/// `input_ptr` must be valid for `input_len` bytes if `input_len > 0`.
#[no_mangle]
pub unsafe extern "C" fn disk_engine_scan_path_ffi(
    input_ptr: *const u8,
    input_len: usize,
) -> SaleemFfiBuffer {
    ffi_execute_json(input_ptr, input_len, |req: DiskScanRequest| {
        let token = CancellationToken::new();
        scan_directory_with_progress(&req, &token, |_, _, _, _, _| {})
    })
}

/// Executes a synchronous duplicate scan over the C ABI.
///
/// # Safety
/// `input_ptr` must be valid for `input_len` bytes if `input_len > 0`.
#[no_mangle]
pub unsafe extern "C" fn disk_engine_find_duplicates_ffi(
    input_ptr: *const u8,
    input_len: usize,
) -> SaleemFfiBuffer {
    ffi_execute_json(input_ptr, input_len, |req: DuplicateScanRequest| {
        let token = CancellationToken::new();
        find_duplicates_with_progress(&req, &token, |_, _, _, _, _| {})
    })
}

/// Executes verified duplicate removal over the C ABI.
///
/// # Safety
/// `input_ptr` must be valid for `input_len` bytes if `input_len > 0`.
#[no_mangle]
pub unsafe extern "C" fn disk_engine_remove_duplicates_ffi(
    input_ptr: *const u8,
    input_len: usize,
) -> SaleemFfiBuffer {
    ffi_execute_json(
        input_ptr,
        input_len,
        |selections: Vec<DuplicateRemovalGroupSelection>| {
            verify_and_remove_duplicate_selections(&selections)
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use saleem_core::ffi::consume_ffi_buffer;

    #[test]
    fn test_disk_engine_abi_version_and_health() {
        assert_eq!(disk_engine_abi_version(), SALEEM_ABI_VERSION);
        let req = br#"{}"#;
        unsafe {
            let buf = disk_engine_health_ffi(req.as_ptr(), req.len());
            let info: EngineHealthInfo =
                consume_ffi_buffer(buf, disk_engine_free_buffer).expect("FFI health check failed");
            assert_eq!(info.engine_name, "SALEEM Disk Engine");
            assert_eq!(info.abi_version, SALEEM_ABI_VERSION);
        }
    }

    #[test]
    fn test_list_volumes_returns_real_drives() {
        let volumes = list_volumes().expect("list_volumes should succeed on Windows");
        assert!(
            !volumes.is_empty(),
            "At least C:\\ volume must be discovered"
        );
        assert!(volumes
            .iter()
            .any(|v| v.mount_point.eq_ignore_ascii_case("C:\\")));
    }

    #[test]
    fn test_disk_engine_scan_path_ffi_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("sample.bin"), vec![42u8; 8192]).unwrap();
        let req = DiskScanRequest {
            target_path: dir.path().to_string_lossy().into_owned(),
            max_top_files: Some(10),
            max_top_folders: Some(10),
            exclude_patterns: None,
        };
        let req_json = serde_json::to_vec(&req).unwrap();
        unsafe {
            let buf = disk_engine_scan_path_ffi(req_json.as_ptr(), req_json.len());
            let res: DiskScanResult =
                consume_ffi_buffer(buf, disk_engine_free_buffer).expect("FFI scan failed");
            assert_eq!(res.files_count, 1);
            assert_eq!(res.total_logical_bytes, 8192);
        }
    }
}
