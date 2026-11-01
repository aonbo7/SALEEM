use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EngineHealthInfo {
    pub engine_name: String,
    pub engine_version: String,
    pub abi_version: u32,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeInfo {
    pub mount_point: String,
    pub volume_label: String,
    pub file_system: String,
    pub total_bytes: u64,
    pub free_bytes: u64,
    pub used_bytes: u64,
    pub usage_percent: f64,
    pub drive_type: String,
    pub is_removable: bool,
    pub is_ready: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EmptyRequest {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskScanRequest {
    pub target_path: String,
    pub max_top_files: Option<usize>,
    pub max_top_folders: Option<usize>,
    pub exclude_patterns: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannedFileEntry {
    pub path: String,
    pub file_name: String,
    pub parent_dir: String,
    pub extension: String,
    pub category: String,
    pub logical_size: u64,
    pub allocated_size: u64,
    pub modified_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannedFolderSummary {
    pub path: String,
    pub folder_name: String,
    pub logical_size: u64,
    pub allocated_size: u64,
    pub files_count: u64,
    pub folders_count: u64,
    pub is_partial: bool,
    pub percentage_of_root: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionStat {
    pub extension: String,
    pub category: String,
    pub files_count: u64,
    pub logical_size: u64,
    pub allocated_size: u64,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryStat {
    pub category: String,
    pub files_count: u64,
    pub logical_size: u64,
    pub allocated_size: u64,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskScanResult {
    pub scan_id: String,
    pub target_path: String,
    pub status: String,
    pub started_at: String,
    pub completed_at: String,
    pub duration_ms: u64,
    pub total_logical_bytes: u64,
    pub total_allocated_bytes: u64,
    pub files_count: u64,
    pub folders_count: u64,
    pub errors_count: u64,
    pub reparse_points_skipped: u64,
    pub is_partial: bool,
    pub sample_inaccessible_paths: Vec<String>,
    pub immediate_children: Vec<ScannedFolderSummary>,
    pub top_folders: Vec<ScannedFolderSummary>,
    pub top_files: Vec<ScannedFileEntry>,
    pub extension_breakdown: Vec<ExtensionStat>,
    pub category_breakdown: Vec<CategoryStat>,
}

// ============================================================================
// Phase 2 — Duplicate Finder Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateScanRequest {
    pub target_path: String,
    pub min_file_size_bytes: Option<u64>,
    pub max_groups: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateFileMember {
    pub path: String,
    pub file_name: String,
    pub parent_dir: String,
    pub modified_at: String,
    pub is_hardlink_alias: bool,
    pub hardlink_count: u32,
    pub suggested_keep: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateGroup {
    pub group_id: String,
    pub sha256: String,
    pub extension: String,
    pub category: String,
    pub file_size_bytes: u64,
    pub wasted_bytes: u64,
    pub copies: Vec<DuplicateFileMember>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateScanResult {
    pub scan_id: String,
    pub target_path: String,
    pub status: String,
    pub started_at: String,
    pub completed_at: String,
    pub duration_ms: u64,
    pub files_evaluated: u64,
    pub candidate_size_groups: u64,
    pub partial_hashes_computed: u64,
    pub full_hashes_computed: u64,
    pub hardlinks_flagged: u64,
    pub errors_count: u64,
    pub is_partial: bool,
    pub total_duplicate_groups: usize,
    pub total_duplicate_files: usize,
    pub total_reclaimable_bytes: u64,
    pub groups: Vec<DuplicateGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateRemovalGroupSelection {
    pub group_sha256: String,
    pub file_size_bytes: u64,
    pub retained_reference_path: String,
    pub delete_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateRemovalReport {
    pub job_id: String,
    pub succeeded_count: usize,
    pub failed_count: usize,
    pub bytes_reclaimed: u64,
    pub deleted_paths: Vec<String>,
    pub failed_items: Vec<(String, String)>,
}
