use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InstalledAppEntry {
    pub id: String,
    pub display_name: String,
    pub publisher: String,
    pub display_version: String,
    pub install_date: Option<String>,
    pub install_location: Option<String>,
    pub uninstall_string: Option<String>,
    pub has_uninstaller: bool,
    pub size_bytes: u64,
    /// True if `size_bytes` comes from Registry `EstimatedSize` heuristic rather than exact manifest/disk scan.
    pub is_size_estimated: bool,
    /// "System (x64)", "System (x86)", "Current User", or "Game Library"
    pub scope: String,
    /// "Windows Registry", "Steam", "Epic Games", or "Xbox / Game Folder"
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LeftoverCandidate {
    pub id: String,
    pub folder_name: String,
    pub path: String,
    /// "AppData\\Roaming", "AppData\\Local", "AppData\\LocalLow", "ProgramData", or "Sandbox"
    pub scope_label: String,
    pub logical_size_bytes: u64,
    pub allocated_size_bytes: u64,
    pub file_count: usize,
    pub last_modified_at: String,
    pub age_days: u64,
    /// "High", "Medium", or "Low"
    pub confidence: String,
    /// Bilingual explanation of why this folder was flagged and scored at this confidence level.
    pub reason_en: String,
    pub reason_ar: String,
    /// True if the folder appears to contain Saved Games, user profiles, or configuration files.
    pub contains_saved_games_or_user_data: bool,
    /// True if any file inside the folder is currently locked by an active OS handle.
    pub has_locked_files: bool,
    /// True only when `confidence == "High"` AND `!contains_saved_games_or_user_data` AND `!has_locked_files`.
    pub recommended_select: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryScanRequest {
    /// Optional custom root directory for deterministic unit testing of leftover discovery.
    #[serde(default)]
    pub custom_sandbox_root: Option<String>,
    /// Optional override list of installed app names for deterministic unit testing.
    #[serde(default)]
    pub mock_installed_apps: Option<Vec<InstalledAppEntry>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryScanResult {
    pub scan_id: String,
    pub started_at: String,
    pub completed_at: String,
    pub duration_ms: u64,
    pub installed_apps: Vec<InstalledAppEntry>,
    pub leftover_candidates: Vec<LeftoverCandidate>,
    pub total_apps_count: usize,
    pub total_games_count: usize,
    pub total_leftovers_count: usize,
    pub high_confidence_leftovers_count: usize,
    pub total_leftover_bytes: u64,
    pub high_confidence_reclaimable_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeftoverRemovalItemRequest {
    pub path: String,
    pub expected_file_count: usize,
    pub expected_logical_bytes: u64,
    pub confidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeftoverRemovalRequest {
    pub scan_id: String,
    pub items: Vec<LeftoverRemovalItemRequest>,
    pub user_confirmed: bool,
    /// Optional custom sandbox root for unit tests.
    #[serde(default)]
    pub custom_sandbox_root: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeftoverRemovalOutcome {
    pub path: String,
    pub status: String,
    pub reason: String,
    pub bytes_reclaimed: u64,
    pub files_removed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeftoverRemovalReport {
    pub job_id: String,
    pub scan_id: String,
    pub started_at: String,
    pub completed_at: String,
    pub total_requested: usize,
    pub succeeded_count: usize,
    pub skipped_or_failed_count: usize,
    pub bytes_reclaimed: u64,
    pub outcomes: Vec<LeftoverRemovalOutcome>,
}
