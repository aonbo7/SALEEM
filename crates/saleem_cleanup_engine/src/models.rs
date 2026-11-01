use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CleanupEngineHealth {
    pub engine_name: String,
    pub engine_version: String,
    pub abi_version: u32,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EmptyRequest {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupRuleDefinition {
    pub id: String,
    pub name_en: String,
    pub name_ar: String,
    pub category: String,
    pub risk_level: String, // "Low" | "Medium"
    pub enabled_by_default: bool,
    pub allowed_scopes: Vec<String>,
    pub file_extensions: Vec<String>, // empty means all files inside allowed_scopes
    pub file_prefixes: Vec<String>,   // e.g. ["thumbcache_"]
    pub min_age_hours: u64,
    pub exclude_locked_files: bool,
    pub supports_quarantine_backup: bool,
    pub reason_en: String,
    pub reason_ar: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupScanRequest {
    /// Optional list of rule IDs to enable. If None, uses all rules where `enabled_by_default == true`.
    pub enabled_rule_ids: Option<Vec<String>>,
    /// Optional override root directory for sandboxed unit/integration tests.
    pub custom_sandbox_root: Option<String>,
    /// Maximum number of individual candidate items to return in the plan preview.
    pub max_candidates: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupCandidateItem {
    pub candidate_id: String,
    pub rule_id: String,
    pub rule_name_en: String,
    pub rule_name_ar: String,
    pub category: String,
    pub risk_level: String,
    pub path: String,
    pub file_name: String,
    pub parent_dir: String,
    pub logical_size: u64,
    pub allocated_size: u64,
    pub modified_at: String,
    pub age_hours: u64,
    pub is_locked: bool,
    pub safe_to_clean: bool,
    pub reason_en: String,
    pub reason_ar: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupRuleSummary {
    pub rule_id: String,
    pub name_en: String,
    pub name_ar: String,
    pub category: String,
    pub risk_level: String,
    pub enabled_by_default: bool,
    pub candidates_count: usize,
    pub locked_skipped_count: usize,
    pub total_logical_bytes: u64,
    pub total_allocated_bytes: u64,
    pub reason_en: String,
    pub reason_ar: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupPlanResult {
    pub plan_id: String,
    pub pipeline_stage: String, // "PreviewReady"
    pub started_at: String,
    pub completed_at: String,
    pub duration_ms: u64,
    pub total_candidates: usize,
    pub total_locked_skipped: usize,
    pub total_reclaimable_bytes: u64,
    pub total_allocated_bytes: u64,
    pub errors_count: u64,
    pub rule_summaries: Vec<CleanupRuleSummary>,
    pub candidates: Vec<CleanupCandidateItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupExecutionItemRequest {
    pub path: String,
    pub rule_id: String,
    pub expected_size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupExecutionRequest {
    pub plan_id: String,
    pub selected_items: Vec<CleanupExecutionItemRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupItemExecutionOutcome {
    pub path: String,
    pub rule_id: String,
    pub action: String, // "DELETE_SAFE_CANDIDATE"
    pub status: String, // "VERIFIED_REMOVED" | "SKIPPED_LOCKED" | "SKIPPED_MODIFIED" | "REJECTED_POLICY" | "FAILED_IO"
    pub bytes_reclaimed: u64,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupExecutionReport {
    pub job_id: String,
    pub plan_id: String,
    pub pipeline_stage: String, // "CompletedAndVerified"
    pub started_at: String,
    pub completed_at: String,
    pub duration_ms: u64,
    pub total_requested: usize,
    pub succeeded_count: usize,
    pub skipped_or_failed_count: usize,
    pub bytes_reclaimed: u64,
    pub outcomes: Vec<CleanupItemExecutionOutcome>,
}
