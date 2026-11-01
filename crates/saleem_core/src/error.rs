use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Structured error categories matching Section 17 of MASTER_PROJECT_SPECIFICATION.md.
#[derive(Debug, Error, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "code", content = "details")]
pub enum SaleemError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Path not found: {0}")]
    PathNotFound(String),

    #[error("Access denied: {0}")]
    AccessDenied(String),

    #[error("Sharing violation (file in use): {0}")]
    SharingViolation(String),

    #[error("File changed since preview (TOCTOU protection triggered): {0}")]
    FileChangedSincePreview(String),

    #[error("Operation blocked on protected system path: {0}")]
    ProtectedSystemPath(String),

    #[error("Unsupported file type: {0}")]
    UnsupportedFileType(String),

    #[error("Scan or operation was cancelled by user")]
    ScanCancelled,

    #[error("Partial scan completed with inaccessible items: {0}")]
    PartialScan(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Rule validation failed: {0}")]
    RuleValidationFailed(String),

    #[error("Signature verification unavailable: {0}")]
    SignatureVerificationUnavailable(String),

    #[error("Quarantine integrity check failed: {0}")]
    QuarantineIntegrityFailed(String),

    #[error("Native engine FFI error: {0}")]
    FfiError(String),

    #[error("I/O error: {0}")]
    IoError(String),
}

pub type SaleemResult<T> = Result<T, SaleemError>;

/// User-facing structured error payload without internal stack traces.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserErrorPayload {
    pub error_code: String,
    pub message_en: String,
    pub message_ar: String,
    pub technical_hint: Option<String>,
}

impl SaleemError {
    pub fn code_name(&self) -> &'static str {
        match self {
            SaleemError::InvalidInput(_) => "InvalidInput",
            SaleemError::PathNotFound(_) => "PathNotFound",
            SaleemError::AccessDenied(_) => "AccessDenied",
            SaleemError::SharingViolation(_) => "SharingViolation",
            SaleemError::FileChangedSincePreview(_) => "FileChangedSincePreview",
            SaleemError::ProtectedSystemPath(_) => "ProtectedSystemPath",
            SaleemError::UnsupportedFileType(_) => "UnsupportedFileType",
            SaleemError::ScanCancelled => "ScanCancelled",
            SaleemError::PartialScan(_) => "PartialScan",
            SaleemError::DatabaseError(_) => "DatabaseError",
            SaleemError::RuleValidationFailed(_) => "RuleValidationFailed",
            SaleemError::SignatureVerificationUnavailable(_) => "SignatureVerificationUnavailable",
            SaleemError::QuarantineIntegrityFailed(_) => "QuarantineIntegrityFailed",
            SaleemError::FfiError(_) => "FfiError",
            SaleemError::IoError(_) => "IoError",
        }
    }

    pub fn to_user_payload(&self) -> UserErrorPayload {
        match self {
            SaleemError::InvalidInput(d) => UserErrorPayload {
                error_code: self.code_name().into(),
                message_en: "The provided input or parameter is invalid.".into(),
                message_ar: "المدخلات أو المعلمات المقدمة غير صالحة.".into(),
                technical_hint: Some(d.clone()),
            },
            SaleemError::PathNotFound(d) => UserErrorPayload {
                error_code: self.code_name().into(),
                message_en: "The requested file or folder path could not be found.".into(),
                message_ar: "لم يتم العثور على مسار الملف أو المجلد المطلوب.".into(),
                technical_hint: Some(d.clone()),
            },
            SaleemError::AccessDenied(d) => UserErrorPayload {
                error_code: self.code_name().into(),
                message_en: "Windows denied access to this path due to insufficient permissions.".into(),
                message_ar: "رفض نظام Windows الوصول إلى هذا المسار لعدم توفر الصلاحيات الكافية.".into(),
                technical_hint: Some(d.clone()),
            },
            SaleemError::SharingViolation(d) => UserErrorPayload {
                error_code: self.code_name().into(),
                message_en: "The file is currently locked or in use by another process.".into(),
                message_ar: "الملف مقفل حالياً أو قيد الاستخدام بواسطة عملية أخرى.".into(),
                technical_hint: Some(d.clone()),
            },
            SaleemError::FileChangedSincePreview(d) => UserErrorPayload {
                error_code: self.code_name().into(),
                message_en: "Safety check aborted action: the file changed after preview.".into(),
                message_ar: "تم إيقاف العملية حفاظاً على السلامة: تغير الملف بعد المعاينة.".into(),
                technical_hint: Some(d.clone()),
            },
            SaleemError::ProtectedSystemPath(d) => UserErrorPayload {
                error_code: self.code_name().into(),
                message_en: "This path belongs to critical Windows system components and cannot be modified or deleted.".into(),
                message_ar: "ينتمي هذا المسار إلى ملفات نظام Windows الحرجة ولا يُسمح بتعديله أو حذفه.".into(),
                technical_hint: Some(d.clone()),
            },
            SaleemError::UnsupportedFileType(d) => UserErrorPayload {
                error_code: self.code_name().into(),
                message_en: "This file format is not supported for this operation.".into(),
                message_ar: "نوع هذا الملف غير مدعوم لهذه العملية.".into(),
                technical_hint: Some(d.clone()),
            },
            SaleemError::ScanCancelled => UserErrorPayload {
                error_code: self.code_name().into(),
                message_en: "The operation was safely cancelled.".into(),
                message_ar: "تم إلغاء العملية بأمان.".into(),
                technical_hint: None,
            },
            SaleemError::PartialScan(d) => UserErrorPayload {
                error_code: self.code_name().into(),
                message_en: "Scan completed partially; some folders required higher permissions.".into(),
                message_ar: "اكتمل الفحص جزئياً؛ بعض المجلدات تطلبت صلاحيات أعلى.".into(),
                technical_hint: Some(d.clone()),
            },
            SaleemError::DatabaseError(d) => UserErrorPayload {
                error_code: self.code_name().into(),
                message_en: "An error occurred while accessing the local SQLite database.".into(),
                message_ar: "حدث خطأ أثناء الوصول إلى قاعدة البيانات المحلية.".into(),
                technical_hint: Some(d.clone()),
            },
            SaleemError::RuleValidationFailed(d) => UserErrorPayload {
                error_code: self.code_name().into(),
                message_en: "The rule definition failed safety or schema validation.".into(),
                message_ar: "فشل التحقق من سلامة أو بنية القاعدة.".into(),
                technical_hint: Some(d.clone()),
            },
            SaleemError::SignatureVerificationUnavailable(d) => UserErrorPayload {
                error_code: self.code_name().into(),
                message_en: "Digital signature verification could not be completed for this file.".into(),
                message_ar: "تعذر إكمال التحقق من التوقيع الرقمي لهذا الملف.".into(),
                technical_hint: Some(d.clone()),
            },
            SaleemError::QuarantineIntegrityFailed(d) => UserErrorPayload {
                error_code: self.code_name().into(),
                message_en: "Quarantine SHA-256 integrity verification failed.".into(),
                message_ar: "فشل التحقق من سلامة الملف (SHA-256) في نظام العزل.".into(),
                technical_hint: Some(d.clone()),
            },
            SaleemError::FfiError(d) => UserErrorPayload {
                error_code: self.code_name().into(),
                message_en: "An internal native engine communication error occurred.".into(),
                message_ar: "حدث خطأ أثناء التواصل مع المحرك الداخلي.".into(),
                technical_hint: Some(d.clone()),
            },
            SaleemError::IoError(d) => UserErrorPayload {
                error_code: self.code_name().into(),
                message_en: "A filesystem I/O error occurred.".into(),
                message_ar: "حدث خطأ أثناء قراءة أو كتابة الملفات.".into(),
                technical_hint: Some(d.clone()),
            },
        }
    }
}

impl From<std::io::Error> for SaleemError {
    fn from(err: std::io::Error) -> Self {
        use std::io::ErrorKind;
        match err.kind() {
            ErrorKind::NotFound => SaleemError::PathNotFound(err.to_string()),
            ErrorKind::PermissionDenied => SaleemError::AccessDenied(err.to_string()),
            _ => {
                // Win32 ERROR_SHARING_VIOLATION is 32, ERROR_LOCK_VIOLATION is 33
                if let Some(code) = err.raw_os_error() {
                    if code == 32 || code == 33 {
                        return SaleemError::SharingViolation(err.to_string());
                    }
                    if code == 5 {
                        return SaleemError::AccessDenied(err.to_string());
                    }
                }
                SaleemError::IoError(err.to_string())
            }
        }
    }
}

impl From<rusqlite::Error> for SaleemError {
    fn from(err: rusqlite::Error) -> Self {
        SaleemError::DatabaseError(err.to_string())
    }
}
