use crate::error::{SaleemError, SaleemResult};
use serde::{de::DeserializeOwned, Serialize};
use std::panic::{catch_unwind, AssertUnwindSafe};

/// Current C-ABI contract version shared across `SALEEM.exe` and `SALEEM.*.dll`.
pub const SALEEM_ABI_VERSION: u32 = 1;

/// Status codes returned across C-ABI boundaries.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaleemFfiStatus {
    Ok = 0,
    InvalidInput = -1,
    SerializationError = -2,
    ExecutionError = -3,
    Cancelled = -4,
    InternalPanic = -99,
}

/// Stable `#[repr(C)]` heap-allocated byte buffer returned by engine DLLs.
/// Must be freed exclusively by calling the exporting DLL's `*_free_buffer` function.
#[repr(C)]
#[derive(Debug)]
pub struct SaleemFfiBuffer {
    pub ptr: *mut u8,
    pub len: usize,
    pub cap: usize,
    pub status_code: i32,
}

impl SaleemFfiBuffer {
    pub fn from_string(payload: String, status: SaleemFfiStatus) -> Self {
        let mut bytes = payload.into_bytes();
        let ptr = bytes.as_mut_ptr();
        let len = bytes.len();
        let cap = bytes.capacity();
        std::mem::forget(bytes);
        Self {
            ptr,
            len,
            cap,
            status_code: status as i32,
        }
    }

    pub fn null_error(status: SaleemFfiStatus) -> Self {
        Self {
            ptr: std::ptr::null_mut(),
            len: 0,
            cap: 0,
            status_code: status as i32,
        }
    }
}

/// Frees a `SaleemFfiBuffer` that was allocated in the current module/DLL's heap.
///
/// # Safety
/// `buf` must have been created by `SaleemFfiBuffer::from_string` within the same allocator.
pub unsafe fn ffi_free_buffer_impl(buf: SaleemFfiBuffer) {
    if !buf.ptr.is_null() && buf.cap > 0 {
        let _ = Vec::from_raw_parts(buf.ptr, buf.len, buf.cap);
    }
}

/// Executes a typed Rust handler across an `extern "C"` boundary with full panic containment (`catch_unwind`).
///
/// # Safety
/// If `input_len > 0`, `input_ptr` must point to a valid byte slice of length `input_len`.
pub unsafe fn ffi_execute_json<Req, Resp, F>(
    input_ptr: *const u8,
    input_len: usize,
    handler: F,
) -> SaleemFfiBuffer
where
    Req: DeserializeOwned,
    Resp: Serialize,
    F: FnOnce(Req) -> SaleemResult<Resp>,
{
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        if input_ptr.is_null() && input_len > 0 {
            return SaleemFfiBuffer::from_string(
                r#"{"error":"Null input pointer with non-zero length"}"#.to_string(),
                SaleemFfiStatus::InvalidInput,
            );
        }

        let input_slice = if input_len == 0 {
            &[][..]
        } else {
            std::slice::from_raw_parts(input_ptr, input_len)
        };

        let req: Req = match serde_json::from_slice(input_slice) {
            Ok(r) => r,
            Err(e) => {
                let err_json = serde_json::json!({
                    "error": format!("Invalid JSON request: {}", e)
                })
                .to_string();
                return SaleemFfiBuffer::from_string(err_json, SaleemFfiStatus::InvalidInput);
            }
        };

        match handler(req) {
            Ok(resp) => match serde_json::to_string(&resp) {
                Ok(json) => SaleemFfiBuffer::from_string(json, SaleemFfiStatus::Ok),
                Err(e) => SaleemFfiBuffer::from_string(
                    serde_json::json!({ "error": e.to_string() }).to_string(),
                    SaleemFfiStatus::SerializationError,
                ),
            },
            Err(SaleemError::ScanCancelled) => SaleemFfiBuffer::from_string(
                serde_json::json!({ "error": "ScanCancelled" }).to_string(),
                SaleemFfiStatus::Cancelled,
            ),
            Err(err) => {
                let err_json = serde_json::to_string(&err)
                    .unwrap_or_else(|_| format!(r#"{{"error":"{}"}}"#, err));
                SaleemFfiBuffer::from_string(err_json, SaleemFfiStatus::ExecutionError)
            }
        }
    }));

    match outcome {
        Ok(buf) => buf,
        Err(_) => SaleemFfiBuffer::from_string(
            r#"{"error":"Engine panic caught safely at C-ABI boundary"}"#.to_string(),
            SaleemFfiStatus::InternalPanic,
        ),
    }
}

/// Reads a `SaleemFfiBuffer` returned by a DLL, copies its UTF-8 string, and immediately frees the buffer
/// using the DLL's exported `free_fn`.
///
/// # Safety
/// `free_fn` must be the matching deallocator from the DLL that produced `buf`.
pub unsafe fn consume_ffi_buffer<T: DeserializeOwned>(
    buf: SaleemFfiBuffer,
    free_fn: unsafe extern "C" fn(SaleemFfiBuffer),
) -> SaleemResult<T> {
    let status = buf.status_code;
    let payload_str = if !buf.ptr.is_null() && buf.len > 0 {
        let slice = std::slice::from_raw_parts(buf.ptr, buf.len);
        String::from_utf8_lossy(slice).into_owned()
    } else {
        String::new()
    };

    free_fn(buf);

    if status == SaleemFfiStatus::Ok as i32 {
        serde_json::from_str::<T>(&payload_str).map_err(|e| {
            SaleemError::FfiError(format!("Failed to decode DLL response JSON: {}", e))
        })
    } else if status == SaleemFfiStatus::Cancelled as i32 {
        Err(SaleemError::ScanCancelled)
    } else if status == SaleemFfiStatus::InternalPanic as i32 {
        Err(SaleemError::FfiError(format!(
            "Engine DLL caught an internal panic: {}",
            payload_str
        )))
    } else {
        // Attempt to deserialize structured SaleemError first
        if let Ok(structured_err) = serde_json::from_str::<SaleemError>(&payload_str) {
            return Err(structured_err);
        }
        Err(SaleemError::FfiError(format!(
            "DLL status {}: {}",
            status, payload_str
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    struct PingReq {
        value: i32,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct PingResp {
        doubled: i32,
    }

    unsafe extern "C" fn local_free(buf: SaleemFfiBuffer) {
        ffi_free_buffer_impl(buf);
    }

    #[test]
    fn test_ffi_roundtrip_success() {
        let req_bytes = br#"{"value": 21}"#;
        unsafe {
            let buf = ffi_execute_json(req_bytes.as_ptr(), req_bytes.len(), |req: PingReq| {
                Ok(PingResp {
                    doubled: req.value * 2,
                })
            });
            assert_eq!(buf.status_code, SaleemFfiStatus::Ok as i32);
            let decoded: PingResp = consume_ffi_buffer(buf, local_free).unwrap();
            assert_eq!(decoded, PingResp { doubled: 42 });
        }
    }

    #[test]
    fn test_ffi_null_pointer_and_invalid_json_rejected() {
        unsafe {
            let buf = ffi_execute_json::<PingReq, PingResp, _>(std::ptr::null(), 10, |_| {
                Ok(PingResp { doubled: 0 })
            });
            assert_eq!(buf.status_code, SaleemFfiStatus::InvalidInput as i32);
            let res: SaleemResult<PingResp> = consume_ffi_buffer(buf, local_free);
            assert!(res.is_err());

            let bad_json = b"{not_valid_json";
            let buf2 =
                ffi_execute_json::<PingReq, PingResp, _>(bad_json.as_ptr(), bad_json.len(), |_| {
                    Ok(PingResp { doubled: 0 })
                });
            assert_eq!(buf2.status_code, SaleemFfiStatus::InvalidInput as i32);
            let res2: SaleemResult<PingResp> = consume_ffi_buffer(buf2, local_free);
            assert!(res2.is_err());
        }
    }

    #[test]
    fn test_ffi_catches_panic_without_unwinding() {
        let req_bytes = br#"{"value": 1}"#;
        unsafe {
            let buf = ffi_execute_json::<PingReq, PingResp, _>(
                req_bytes.as_ptr(),
                req_bytes.len(),
                |_req| {
                    panic!("Simulated parser panic inside native engine");
                },
            );
            assert_eq!(buf.status_code, SaleemFfiStatus::InternalPanic as i32);
            let res: SaleemResult<PingResp> = consume_ffi_buffer(buf, local_free);
            assert!(matches!(res, Err(SaleemError::FfiError(_))));
        }
    }
}
