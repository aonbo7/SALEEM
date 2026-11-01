use crate::models::CleanupRuleDefinition;
use std::path::PathBuf;

/// Returns the built-in declarative SALEEM Cleanup Rules.
/// Every rule strictly targets non-critical user-scope temporary/cache/diagnostic directories
/// and never touches cookies, saved passwords, browser history, or protected OS components.
pub fn default_cleanup_rules() -> Vec<CleanupRuleDefinition> {
    vec![
        CleanupRuleDefinition {
            id: "CLEAN-TEMP-USER-001".to_string(),
            name_en: "User Temporary Files (%LOCALAPPDATA%\\Temp)".to_string(),
            name_ar: "الملفات المؤقتة للمستخدم (%LOCALAPPDATA%\\Temp)".to_string(),
            category: "TemporaryFiles".to_string(),
            risk_level: "Low".to_string(),
            enabled_by_default: true,
            allowed_scopes: vec!["%LOCALAPPDATA%\\Temp".to_string()],
            file_extensions: vec![],
            file_prefixes: vec![],
            min_age_hours: 1,
            exclude_locked_files: true,
            supports_quarantine_backup: false,
            reason_en: "Temporary scratch files created by user applications that are not currently locked by any running process.".to_string(),
            reason_ar: "ملفات مؤقتة أنشأتها البرامج في مجلد المستخدم المؤقت وليست مفتوحة أو قيد الاستخدام حالياً.".to_string(),
        },
        CleanupRuleDefinition {
            id: "CLEAN-CACHE-THUMB-002".to_string(),
            name_en: "Windows Explorer Thumbnail Cache".to_string(),
            name_ar: "ذاكرة المصغرات المؤقتة (Thumbnail Cache)".to_string(),
            category: "ApplicationCache".to_string(),
            risk_level: "Low".to_string(),
            enabled_by_default: true,
            allowed_scopes: vec!["%LOCALAPPDATA%\\Microsoft\\Windows\\Explorer".to_string()],
            file_extensions: vec!["db".to_string()],
            file_prefixes: vec!["thumbcache_".to_string()],
            min_age_hours: 0,
            exclude_locked_files: true,
            supports_quarantine_backup: false,
            reason_en: "Unlocked Explorer thumbnail cache databases that Windows automatically rebuilds on demand.".to_string(),
            reason_ar: "قواعد بيانات الصور المصغرة غير المقفلة التي يعيد نظام Windows بناءها تلقائياً عند الحاجة.".to_string(),
        },
        CleanupRuleDefinition {
            id: "CLEAN-CACHE-BROWSER-003".to_string(),
            name_en: "Browser HTTP & Code Caches (Edge / Chrome / Brave)".to_string(),
            name_ar: "الكاش المؤقت للمتصفحات (دون المساس بكلمات المرور أو الجلسات)".to_string(),
            category: "BrowserCache".to_string(),
            risk_level: "Low".to_string(),
            enabled_by_default: true,
            allowed_scopes: vec![
                "%LOCALAPPDATA%\\Microsoft\\Edge\\User Data\\Default\\Cache\\Cache_Data".to_string(),
                "%LOCALAPPDATA%\\Microsoft\\Edge\\User Data\\Default\\Code Cache\\js".to_string(),
                "%LOCALAPPDATA%\\Microsoft\\Edge\\User Data\\Default\\GPUCache".to_string(),
                "%LOCALAPPDATA%\\Google\\Chrome\\User Data\\Default\\Cache\\Cache_Data".to_string(),
                "%LOCALAPPDATA%\\Google\\Chrome\\User Data\\Default\\Code Cache\\js".to_string(),
                "%LOCALAPPDATA%\\Google\\Chrome\\User Data\\Default\\GPUCache".to_string(),
                "%LOCALAPPDATA%\\BraveSoftware\\Brave-Browser\\User Data\\Default\\Cache\\Cache_Data".to_string(),
            ],
            file_extensions: vec![],
            file_prefixes: vec![],
            min_age_hours: 0,
            exclude_locked_files: true,
            supports_quarantine_backup: false,
            reason_en: "Cached web assets and compiled JS/GPU bytecode only. Strictly excludes cookies, history, bookmarks, and saved logins.".to_string(),
            reason_ar: "ملفات الكاش المؤقتة للويب فقط؛ لا يمس الكوكيز أو كلمات المرور أو السجل أو المفضلة إطلاقاً.".to_string(),
        },
        CleanupRuleDefinition {
            id: "CLEAN-SHADER-DX-004".to_string(),
            name_en: "DirectX & GPU Shader Cache".to_string(),
            name_ar: "ذاكرة تظليل البطاقة الرسومية (DirectX & GPU Shader Cache)".to_string(),
            category: "ShaderCache".to_string(),
            risk_level: "Low".to_string(),
            enabled_by_default: true,
            allowed_scopes: vec![
                "%LOCALAPPDATA%\\D3DSCache".to_string(),
                "%LOCALAPPDATA%\\NVIDIA\\DXCache".to_string(),
                "%LOCALAPPDATA%\\AMD\\DxCache".to_string(),
            ],
            file_extensions: vec![],
            file_prefixes: vec![],
            min_age_hours: 0,
            exclude_locked_files: true,
            supports_quarantine_backup: false,
            reason_en: "Compiled DirectX/GPU shader caches that games and graphics drivers regenerate automatically.".to_string(),
            reason_ar: "ملفات تظليل الرسوميات المترجمة التي تعيد الألعاب وتعريفات الشاشة توليدها تلقائياً.".to_string(),
        },
        CleanupRuleDefinition {
            id: "CLEAN-LOGS-WER-005".to_string(),
            name_en: "Windows Error Reporting (WER) Archives".to_string(),
            name_ar: "تقارير أخطاء التطبيقات المؤرشفة (WER Reports)".to_string(),
            category: "DiagnosticLogs".to_string(),
            risk_level: "Low".to_string(),
            enabled_by_default: true,
            allowed_scopes: vec![
                "%LOCALAPPDATA%\\Microsoft\\Windows\\WER\\ReportArchive".to_string(),
                "%LOCALAPPDATA%\\Microsoft\\Windows\\WER\\ReportQueue".to_string(),
            ],
            file_extensions: vec!["wer".to_string(), "txt".to_string(), "log".to_string(), "cab".to_string()],
            file_prefixes: vec![],
            min_age_hours: 0,
            exclude_locked_files: true,
            supports_quarantine_backup: false,
            reason_en: "Archived diagnostic telemetry reports from older application faults.".to_string(),
            reason_ar: "تقارير تشخيص الأعطال السابقة المؤرشفة في مجلد المستخدم.".to_string(),
        },
        CleanupRuleDefinition {
            id: "CLEAN-DUMPS-CRASH-006".to_string(),
            name_en: "User-Mode Application Crash Dumps (*.dmp)".to_string(),
            name_ar: "ملفات تفريغ الذاكرة عند الانهيار (Crash Dumps *.dmp)".to_string(),
            category: "CrashDumps".to_string(),
            risk_level: "Medium".to_string(),
            enabled_by_default: false,
            allowed_scopes: vec!["%LOCALAPPDATA%\\CrashDumps".to_string()],
            file_extensions: vec!["dmp".to_string(), "hdmp".to_string(), "mdmp".to_string()],
            file_prefixes: vec![],
            min_age_hours: 24,
            exclude_locked_files: true,
            supports_quarantine_backup: true,
            reason_en: "Large memory dump files created when an application crashes. Review before deleting if you are actively debugging a crash.".to_string(),
            reason_ar: "ملفات تفريغ ذاكرة كبيرة تنتج عند انهيار أحد البرامج؛ يُنصح بمراجعتها إذا كنت تحلل عطلاً برمجياً حديثاً.".to_string(),
        },
    ]
}

/// Expands `%LOCALAPPDATA%`, `%APPDATA%`, `%TEMP%`, and `%USERPROFILE%` prefixes safely.
pub fn expand_windows_env_scope(raw_scope: &str) -> Option<PathBuf> {
    let mut expanded = raw_scope.to_string();
    for var_name in ["LOCALAPPDATA", "APPDATA", "TEMP", "USERPROFILE"] {
        let token = format!("%{}%", var_name);
        if expanded.to_ascii_uppercase().contains(&token) {
            if let Ok(val) = std::env::var(var_name) {
                let upper = expanded.to_ascii_uppercase();
                if let Some(idx) = upper.find(&token) {
                    let end = idx + token.len();
                    expanded.replace_range(idx..end, val.trim_end_matches('\\'));
                }
            } else {
                return None;
            }
        }
    }
    Some(PathBuf::from(expanded))
}
