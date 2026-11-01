export type LanguageCode = "ar" | "en";
export type ThemeMode = "dark" | "light";

export type NavSectionId =
  | "overview"
  | "storage"
  | "large_files"
  | "duplicates"
  | "cleanup"
  | "applications"
  | "startup"
  | "memory"
  | "sentinel"
  | "quarantine"
  | "history"
  | "settings";

export interface UserErrorPayload {
  error_code: string;
  message_en: string;
  message_ar: string;
  technical_hint?: string | null;
}

export interface DatabaseHealthSummary {
  schema_version: number;
  tables_count: number;
  scans_count: number;
  cleanup_jobs_count: number;
  security_scans_count: number;
  quarantine_active_count: number;
  audit_events_count: number;
}

export interface NativeDllStatus {
  logical_name: string;
  dll_file_name: string;
  resolved_path: string;
  loaded_via: string;
  abi_version: number;
  expected_abi_version: number;
  is_compatible: boolean;
  status: string;
}

export interface VolumeInfo {
  mount_point: string;
  volume_label: string;
  file_system: string;
  total_bytes: number;
  free_bytes: number;
  used_bytes: number;
  usage_percent: number;
  drive_type: string;
  is_removable: boolean;
  is_ready: boolean;
}

export interface AuditEventRecord {
  id: number;
  timestamp: string;
  event_type: string;
  subject: string;
  result: string;
  details_redacted: string;
}

export interface SystemFoundationStatus {
  app_name: string;
  app_version: string;
  current_phase: string;
  ui_language: LanguageCode;
  ui_theme: ThemeMode;
  database: DatabaseHealthSummary;
  engines: NativeDllStatus[];
  volumes: VolumeInfo[];
}

export interface QuickScanPreset {
  id: string;
  label_en: string;
  label_ar: string;
  path: string;
}

export type JobStatus = "Running" | "Completed" | "Cancelled" | "Failed";

export interface JobProgressSnapshot {
  job_id: string;
  job_type: string;
  target: string;
  status: JobStatus;
  files_processed: number;
  folders_processed: number;
  bytes_processed: number;
  current_path: string;
  errors_count: number;
}

export interface ScannedFileEntry {
  path: string;
  file_name: string;
  parent_dir: string;
  extension: string;
  category: string;
  logical_size: number;
  allocated_size: number;
  modified_at: string;
}

export interface ScannedFolderSummary {
  path: string;
  folder_name: string;
  logical_size: number;
  allocated_size: number;
  files_count: number;
  folders_count: number;
  is_partial: boolean;
  percentage_of_root: number;
}

export interface ExtensionStat {
  extension: string;
  category: string;
  files_count: number;
  logical_size: number;
  allocated_size: number;
  percentage: number;
}

export interface CategoryStat {
  category: string;
  files_count: number;
  logical_size: number;
  allocated_size: number;
  percentage: number;
}

export interface DiskScanResult {
  scan_id: string;
  target_path: string;
  status: string;
  started_at: string;
  completed_at: string;
  duration_ms: number;
  total_logical_bytes: number;
  total_allocated_bytes: number;
  files_count: number;
  folders_count: number;
  errors_count: number;
  reparse_points_skipped: number;
  is_partial: boolean;
  sample_inaccessible_paths: string[];
  immediate_children: ScannedFolderSummary[];
  top_folders: ScannedFolderSummary[];
  top_files: ScannedFileEntry[];
  extension_breakdown: ExtensionStat[];
  category_breakdown: CategoryStat[];
}

export interface DuplicateFileMember {
  path: string;
  file_name: string;
  parent_dir: string;
  modified_at: string;
  is_hardlink_alias: boolean;
  hardlink_count: number;
  suggested_keep: boolean;
}

export interface DuplicateGroup {
  group_id: string;
  sha256: string;
  extension: string;
  category: string;
  file_size_bytes: number;
  wasted_bytes: number;
  copies: DuplicateFileMember[];
}

export interface DuplicateScanResult {
  scan_id: string;
  target_path: string;
  status: string;
  started_at: string;
  completed_at: string;
  duration_ms: number;
  files_evaluated: number;
  candidate_size_groups: number;
  partial_hashes_computed: number;
  full_hashes_computed: number;
  hardlinks_flagged: number;
  errors_count: number;
  is_partial: boolean;
  total_duplicate_groups: number;
  total_duplicate_files: number;
  total_reclaimable_bytes: number;
  groups: DuplicateGroup[];
}

export interface DuplicateRemovalGroupSelection {
  group_sha256: string;
  file_size_bytes: number;
  retained_reference_path: string;
  delete_paths: string[];
}

export interface DuplicateRemovalReport {
  job_id: string;
  succeeded_count: number;
  failed_count: number;
  bytes_reclaimed: number;
  deleted_paths: string[];
  failed_items: [string, string][];
}

export interface CleanupRuleDefinition {
  id: string;
  name_en: string;
  name_ar: string;
  category: string;
  risk_level: string;
  enabled_by_default: boolean;
  allowed_scopes: string[];
  file_extensions: string[];
  file_prefixes: string[];
  min_age_hours: number;
  exclude_locked_files: boolean;
  supports_quarantine_backup: boolean;
  reason_en: string;
  reason_ar: string;
}

export interface CleanupCandidateItem {
  candidate_id: string;
  rule_id: string;
  rule_name_en: string;
  rule_name_ar: string;
  category: string;
  risk_level: string;
  path: string;
  file_name: string;
  parent_dir: string;
  logical_size: number;
  allocated_size: number;
  modified_at: string;
  age_hours: number;
  is_locked: boolean;
  safe_to_clean: boolean;
  reason_en: string;
  reason_ar: string;
}

export interface CleanupRuleSummary {
  rule_id: string;
  name_en: string;
  name_ar: string;
  category: string;
  risk_level: string;
  enabled_by_default: boolean;
  candidates_count: number;
  locked_skipped_count: number;
  total_logical_bytes: number;
  total_allocated_bytes: number;
  reason_en: string;
  reason_ar: string;
}

export interface CleanupPlanResult {
  plan_id: string;
  pipeline_stage: string;
  started_at: string;
  completed_at: string;
  duration_ms: number;
  total_candidates: number;
  total_locked_skipped: number;
  total_reclaimable_bytes: number;
  total_allocated_bytes: number;
  errors_count: number;
  rule_summaries: CleanupRuleSummary[];
  candidates: CleanupCandidateItem[];
}

export interface CleanupExecutionItemRequest {
  path: string;
  rule_id: string;
  expected_size_bytes: number;
}

export interface CleanupItemExecutionOutcome {
  path: string;
  rule_id: string;
  action: string;
  status: string;
  bytes_reclaimed: number;
  detail: string;
}

export interface CleanupExecutionReport {
  job_id: string;
  plan_id: string;
  pipeline_stage: string;
  started_at: string;
  completed_at: string;
  duration_ms: number;
  total_requested: number;
  succeeded_count: number;
  skipped_or_failed_count: number;
  bytes_reclaimed: number;
  outcomes: CleanupItemExecutionOutcome[];
}

export interface InstalledAppEntry {
  id: string;
  display_name: string;
  publisher: string;
  display_version: string;
  install_date: string | null;
  install_location: string | null;
  uninstall_string: string | null;
  has_uninstaller: boolean;
  size_bytes: number;
  is_size_estimated: boolean;
  scope: string;
  source: string;
}

export interface LeftoverCandidate {
  id: string;
  folder_name: string;
  path: string;
  scope_label: string;
  logical_size_bytes: number;
  allocated_size_bytes: number;
  file_count: number;
  last_modified_at: string;
  age_days: number;
  confidence: string;
  reason_en: string;
  reason_ar: string;
  contains_saved_games_or_user_data: boolean;
  has_locked_files: boolean;
  recommended_select: boolean;
}

export interface InventoryScanResult {
  scan_id: string;
  started_at: string;
  completed_at: string;
  duration_ms: number;
  installed_apps: InstalledAppEntry[];
  leftover_candidates: LeftoverCandidate[];
  total_apps_count: number;
  total_games_count: number;
  total_leftovers_count: number;
  high_confidence_leftovers_count: number;
  total_leftover_bytes: number;
  high_confidence_reclaimable_bytes: number;
}

export interface LeftoverRemovalItemRequest {
  path: string;
  expected_file_count: number;
  expected_logical_bytes: number;
  confidence: string;
}

export interface LeftoverRemovalOutcome {
  path: string;
  status: string;
  reason: string;
  bytes_reclaimed: number;
  files_removed: number;
}

export interface LeftoverRemovalReport {
  job_id: string;
  scan_id: string;
  started_at: string;
  completed_at: string;
  total_requested: number;
  succeeded_count: number;
  skipped_or_failed_count: number;
  bytes_reclaimed: number;
  outcomes: LeftoverRemovalOutcome[];
}

export interface CloudUpdateConfig {
  supabase_url: string;
  anon_key_configured: boolean;
  anon_key_masked: string;
  current_version: string;
}

export interface CloudReleaseRecord {
  version: string;
  channel: string;
  download_url: string;
  sha256_checksum: string;
  file_size_bytes: number;
  release_notes_ar: string;
  release_notes_en: string;
  is_mandatory: boolean;
  published_at: string;
}

export interface CloudUpdateCheckResult {
  update_available: boolean;
  current_version: string;
  latest_release: CloudReleaseRecord | null;
  checked_at: string;
}

export interface StartupEntryItem {
  id: string;
  name: string;
  command_or_path: string;
  location_label: string;
  source_type: string;
  can_remove_without_admin: boolean;
  impact_estimate: string;
}

export interface SystemMemoryTelemetry {
  memory_load_percent: number;
  total_physical_bytes: number;
  avail_physical_bytes: number;
  used_physical_bytes: number;
  total_pagefile_bytes: number;
  avail_pagefile_bytes: number;
}

export interface StartupBoosterSnapshot {
  memory: SystemMemoryTelemetry;
  startup_items: StartupEntryItem[];
  thumbcache_reclaimable_bytes: number;
  thumbcache_files_count: number;
}

export interface BoostActionOutcome {
  action_id: string;
  status: string;
  message_ar: string;
  message_en: string;
  bytes_freed: number;
}

export interface MemoryEngineSnapshot extends SystemMemoryTelemetry {
  used_pagefile_bytes: number;
  commit_load_percent: number;
  system_cache_bytes: number;
  kernel_total_bytes: number;
  kernel_paged_bytes: number;
  kernel_nonpaged_bytes: number;
  process_count: number;
  thread_count: number;
  handle_count: number;
}

export interface ProcessMemoryEntry {
  pid: number;
  name: string;
  working_set_bytes: number;
  private_bytes: number;
  peak_working_set_bytes: number;
}

export interface ProcessTrimDiff {
  name: string;
  instance_count: number;
  before_bytes: number;
  after_bytes: number;
  freed_bytes: number;
}

export interface MemoryCleanAreas {
  working_sets_cleaned: boolean;
  system_file_cache_flushed: boolean;
  modified_page_list_flushed: boolean;
  standby_list_purged: boolean;
  low_priority_standby_purged: boolean;
  physical_memory_combined: boolean;
  registry_cache_flushed: boolean;
  pages_combined_count: number;
}

export interface MemoryEngineConfig {
  clean_depth: "Light" | "Medium" | "Deep" | string;
  auto_interval_minutes: number;
  auto_threshold_percent: number;
  clean_on_app_startup: boolean;
  launch_app_on_windows_startup: boolean;
  run_in_background: boolean;
  show_notifications: boolean;
}

export interface MemoryCleanReport {
  depth: string;
  before: MemoryEngineSnapshot;
  after: MemoryEngineSnapshot;
  bytes_freed: number;
  working_set_freed_bytes: number;
  standby_and_cache_freed_bytes: number;
  processes_scanned: number;
  processes_trimmed: number;
  areas: MemoryCleanAreas;
  top_freed_processes: ProcessTrimDiff[];
  duration_ms: number;
  completed_at: string;
  trigger_source: string;
}

export interface MemoryEngineStateResponse {
  config: MemoryEngineConfig;
  snapshot: MemoryEngineSnapshot;
  top_processes: ProcessMemoryEntry[];
  last_report: MemoryCleanReport | null;
  dll_status: NativeDllStatus;
}
