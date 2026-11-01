pub mod engine;
pub mod models;
pub mod rules;

pub use engine::{
    is_file_currently_locked, scan_and_plan_cleanup_with_progress, validate_and_execute_cleanup,
};
pub use models::{
    CleanupCandidateItem, CleanupEngineHealth, CleanupExecutionItemRequest, CleanupExecutionReport,
    CleanupExecutionRequest, CleanupItemExecutionOutcome, CleanupPlanResult, CleanupRuleDefinition,
    CleanupRuleSummary, CleanupScanRequest, EmptyRequest,
};
pub use rules::{default_cleanup_rules, expand_windows_env_scope};

use saleem_core::ffi::{
    ffi_execute_json, ffi_free_buffer_impl, SaleemFfiBuffer, SALEEM_ABI_VERSION,
};
use saleem_core::paths::assert_safe_for_modification;
use saleem_core::{CancellationToken, SaleemResult};

pub const ENGINE_NAME: &str = "SALEEM Cleanup Engine";
pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn get_engine_health() -> CleanupEngineHealth {
    CleanupEngineHealth {
        engine_name: ENGINE_NAME.to_string(),
        engine_version: ENGINE_VERSION.to_string(),
        abi_version: SALEEM_ABI_VERSION,
        status: "READY".to_string(),
    }
}

/// Validates whether a candidate path is permissible under SALEEM's cleanup safety rules.
pub fn validate_cleanup_candidate_path(path: &str) -> SaleemResult<String> {
    let safe_path = assert_safe_for_modification(path)?;
    Ok(safe_path.to_string_lossy().into_owned())
}

// ============================================================================
// C-ABI Exports for `SALEEM.CleanupEngine.dll`
// ============================================================================

#[no_mangle]
pub extern "C" fn cleanup_engine_abi_version() -> u32 {
    SALEEM_ABI_VERSION
}

/// Frees a `SaleemFfiBuffer` allocated by `SALEEM.CleanupEngine.dll`.
///
/// # Safety
/// Must only be called with a buffer returned by `SALEEM.CleanupEngine.dll`.
#[no_mangle]
pub unsafe extern "C" fn cleanup_engine_free_buffer(buf: SaleemFfiBuffer) {
    ffi_free_buffer_impl(buf);
}

/// Returns engine health metadata over the C ABI.
///
/// # Safety
/// `input_ptr` must be valid for `input_len` bytes if `input_len > 0`.
#[no_mangle]
pub unsafe extern "C" fn cleanup_engine_health_ffi(
    input_ptr: *const u8,
    input_len: usize,
) -> SaleemFfiBuffer {
    ffi_execute_json(input_ptr, input_len, |_: EmptyRequest| {
        Ok(get_engine_health())
    })
}

/// Returns all declarative cleanup rules over the C ABI.
///
/// # Safety
/// `input_ptr` must be valid for `input_len` bytes if `input_len > 0`.
#[no_mangle]
pub unsafe extern "C" fn cleanup_engine_list_rules_ffi(
    input_ptr: *const u8,
    input_len: usize,
) -> SaleemFfiBuffer {
    ffi_execute_json(input_ptr, input_len, |_: EmptyRequest| {
        Ok(default_cleanup_rules())
    })
}

/// Executes Stages 1–4 (`Scan -> Analyze -> Plan -> Preview`) over the C ABI.
///
/// # Safety
/// `input_ptr` must be valid for `input_len` bytes if `input_len > 0`.
#[no_mangle]
pub unsafe extern "C" fn cleanup_engine_scan_plan_ffi(
    input_ptr: *const u8,
    input_len: usize,
) -> SaleemFfiBuffer {
    ffi_execute_json(input_ptr, input_len, |req: CleanupScanRequest| {
        let token = CancellationToken::new();
        scan_and_plan_cleanup_with_progress(&req, &token, |_, _, _, _, _| {})
    })
}

/// Executes Stages 5–9 (`Confirm -> Validate -> Execute -> Verify -> Report`) over the C ABI.
///
/// # Safety
/// `input_ptr` must be valid for `input_len` bytes if `input_len > 0`.
#[no_mangle]
pub unsafe extern "C" fn cleanup_engine_execute_plan_ffi(
    input_ptr: *const u8,
    input_len: usize,
) -> SaleemFfiBuffer {
    ffi_execute_json(input_ptr, input_len, |req: CleanupExecutionRequest| {
        validate_and_execute_cleanup(&req)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use saleem_core::ffi::consume_ffi_buffer;

    #[test]
    fn test_cleanup_engine_abi_and_health() {
        assert_eq!(cleanup_engine_abi_version(), SALEEM_ABI_VERSION);
        let req = br#"{}"#;
        unsafe {
            let buf = cleanup_engine_health_ffi(req.as_ptr(), req.len());
            let info: CleanupEngineHealth =
                consume_ffi_buffer(buf, cleanup_engine_free_buffer).unwrap();
            assert_eq!(info.engine_name, "SALEEM Cleanup Engine");
        }
    }

    #[test]
    fn test_cleanup_candidate_rejects_system_paths() {
        assert!(validate_cleanup_candidate_path(r"C:\Windows\WinSxS\file.dll").is_err());
        assert!(validate_cleanup_candidate_path(r"C:\Windows\System32\DriverStore").is_err());
        assert!(
            validate_cleanup_candidate_path(r"C:\Users\TestUser\AppData\Local\Temp\test.tmp").is_ok()
        );
    }
}
