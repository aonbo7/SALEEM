use saleem_core::ffi::{
    ffi_execute_json, ffi_free_buffer_impl, SaleemFfiBuffer, SALEEM_ABI_VERSION,
};
use serde::{Deserialize, Serialize};

pub const ENGINE_NAME: &str = "SALEEM Sentinel";
pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DEFAULT_RULES_VERSION: &str = "2026.10.1";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum SentinelRiskLevel {
    Informational,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum SentinelConfidence {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SentinelEngineHealth {
    pub engine_name: String,
    pub engine_version: String,
    pub rules_version: String,
    pub abi_version: u32,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EmptyRequest {}

pub fn get_engine_health() -> SentinelEngineHealth {
    SentinelEngineHealth {
        engine_name: ENGINE_NAME.to_string(),
        engine_version: ENGINE_VERSION.to_string(),
        rules_version: DEFAULT_RULES_VERSION.to_string(),
        abi_version: SALEEM_ABI_VERSION,
        status: "READY".to_string(),
    }
}

// ============================================================================
// C-ABI Exports for `SALEEM.Sentinel.dll`
// ============================================================================

#[no_mangle]
pub extern "C" fn sentinel_abi_version() -> u32 {
    SALEEM_ABI_VERSION
}

/// Frees a `SaleemFfiBuffer` allocated by `SALEEM.Sentinel.dll`.
///
/// # Safety
/// Must only be called with a buffer returned by `SALEEM.Sentinel.dll`.
#[no_mangle]
pub unsafe extern "C" fn sentinel_free_buffer(buf: SaleemFfiBuffer) {
    ffi_free_buffer_impl(buf);
}

/// Returns engine health metadata over the C ABI.
///
/// # Safety
/// `input_ptr` must be valid for `input_len` bytes if `input_len > 0`.
#[no_mangle]
pub unsafe extern "C" fn sentinel_health_ffi(
    input_ptr: *const u8,
    input_len: usize,
) -> SaleemFfiBuffer {
    ffi_execute_json(input_ptr, input_len, |_: EmptyRequest| {
        Ok(get_engine_health())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use saleem_core::ffi::consume_ffi_buffer;

    #[test]
    fn test_sentinel_abi_and_health() {
        assert_eq!(sentinel_abi_version(), SALEEM_ABI_VERSION);
        let req = br#"{}"#;
        unsafe {
            let buf = sentinel_health_ffi(req.as_ptr(), req.len());
            let info: SentinelEngineHealth = consume_ffi_buffer(buf, sentinel_free_buffer).unwrap();
            assert_eq!(info.engine_name, "SALEEM Sentinel");
            assert_eq!(info.abi_version, SALEEM_ABI_VERSION);
        }
    }
}
