import { invoke } from "@tauri-apps/api/core";
import {
  AuditEventRecord,
  BoostActionOutcome,
  CleanupExecutionItemRequest,
  CleanupExecutionReport,
  CleanupPlanResult,
  CleanupRuleDefinition,
  CloudUpdateCheckResult,
  CloudUpdateConfig,
  DiskScanResult,
  DuplicateRemovalGroupSelection,
  DuplicateRemovalReport,
  DuplicateScanResult,
  InventoryScanResult,
  JobProgressSnapshot,
  LanguageCode,
  LeftoverRemovalItemRequest,
  LeftoverRemovalReport,
  MemoryCleanReport,
  MemoryEngineStateResponse,
  QuickScanPreset,
  StartupBoosterSnapshot,
  SystemFoundationStatus,
  ThemeMode,
  VolumeInfo,
} from "../types/saleem";

export async function fetchSystemFoundationStatus(): Promise<SystemFoundationStatus> {
  return await invoke<SystemFoundationStatus>("get_system_foundation_status");
}

export async function fetchMountedVolumes(): Promise<VolumeInfo[]> {
  return await invoke<VolumeInfo[]>("get_volumes_command");
}

export async function fetchQuickScanPresets(): Promise<QuickScanPreset[]> {
  return await invoke<QuickScanPreset[]>("get_quick_scan_presets_command");
}

export async function startDiskScan(
  targetPath: string,
  maxTopFiles = 250,
  maxTopFolders = 100
): Promise<string> {
  return await invoke<string>("start_disk_scan_command", {
    targetPath,
    maxTopFiles,
    maxTopFolders,
  });
}

export async function fetchJobProgress(
  jobId: string
): Promise<JobProgressSnapshot> {
  return await invoke<JobProgressSnapshot>("get_job_progress_command", {
    jobId,
  });
}

export async function cancelActiveJob(jobId: string): Promise<boolean> {
  return await invoke<boolean>("cancel_job_command", { jobId });
}

export async function fetchDiskScanResult(
  jobId: string
): Promise<DiskScanResult | null> {
  return await invoke<DiskScanResult | null>("get_disk_scan_result_command", {
    jobId,
  });
}

export async function fetchLatestDiskScan(): Promise<DiskScanResult | null> {
  return await invoke<DiskScanResult | null>("get_latest_disk_scan_command");
}

export async function startDuplicateScan(
  targetPath: string,
  minSizeBytes = 1024,
  maxGroups = 150
): Promise<string> {
  return await invoke<string>("start_duplicate_scan_command", {
    targetPath,
    minSizeBytes,
    maxGroups,
  });
}

export async function fetchDuplicateScanResult(
  jobId: string
): Promise<DuplicateScanResult | null> {
  return await invoke<DuplicateScanResult | null>(
    "get_duplicate_scan_result_command",
    { jobId }
  );
}

export async function fetchLatestDuplicateScan(): Promise<DuplicateScanResult | null> {
  return await invoke<DuplicateScanResult | null>(
    "get_latest_duplicate_scan_command"
  );
}

export async function executeDuplicateRemoval(
  selections: DuplicateRemovalGroupSelection[]
): Promise<DuplicateRemovalReport> {
  return await invoke<DuplicateRemovalReport>(
    "execute_duplicate_removal_command",
    { selections }
  );
}

export async function fetchCleanupRules(): Promise<CleanupRuleDefinition[]> {
  return await invoke<CleanupRuleDefinition[]>("get_cleanup_rules_command");
}

export async function startCleanupScan(
  enabledRuleIds?: string[]
): Promise<string> {
  return await invoke<string>("start_cleanup_scan_command", {
    enabledRuleIds: enabledRuleIds ?? null,
  });
}

export async function fetchCleanupPlanResult(
  jobId: string
): Promise<CleanupPlanResult | null> {
  return await invoke<CleanupPlanResult | null>(
    "get_cleanup_plan_result_command",
    { jobId }
  );
}

export async function fetchLatestCleanupPlan(): Promise<CleanupPlanResult | null> {
  return await invoke<CleanupPlanResult | null>(
    "get_latest_cleanup_plan_command"
  );
}

export async function executeCleanupPlan(
  planId: string,
  selectedItems: CleanupExecutionItemRequest[]
): Promise<CleanupExecutionReport> {
  return await invoke<CleanupExecutionReport>("execute_cleanup_plan_command", {
    planId,
    selectedItems,
  });
}

export async function startInventoryScan(): Promise<string> {
  return await invoke<string>("start_inventory_scan_command");
}

export async function fetchInventoryScanResult(
  jobId: string
): Promise<InventoryScanResult | null> {
  return await invoke<InventoryScanResult | null>(
    "get_inventory_scan_result_command",
    { jobId }
  );
}

export async function fetchLatestInventoryResult(): Promise<InventoryScanResult | null> {
  return await invoke<InventoryScanResult | null>(
    "get_latest_inventory_result_command"
  );
}

export async function executeLeftoverRemoval(
  scanId: string,
  items: LeftoverRemovalItemRequest[]
): Promise<LeftoverRemovalReport> {
  return await invoke<LeftoverRemovalReport>(
    "execute_leftover_removal_command",
    { scanId, items }
  );
}

export async function launchAppUninstaller(
  appName: string,
  source: string,
  uninstallString: string | null
): Promise<string> {
  return await invoke<string>("launch_app_uninstaller_command", {
    appName,
    source,
    uninstallString,
  });
}

export async function openPathInExplorer(path: string): Promise<void> {
  await invoke<void>("open_in_explorer_command", { path });
}

export async function fetchAuditHistory(
  limit = 50
): Promise<AuditEventRecord[]> {
  return await invoke<AuditEventRecord[]>("get_audit_history_command", {
    limit,
  });
}

export async function updateUiPreferences(
  language: LanguageCode,
  theme: ThemeMode
): Promise<void> {
  await invoke<void>("update_ui_preferences_command", { language, theme });
}

export async function fetchCloudUpdateConfig(): Promise<CloudUpdateConfig> {
  return await invoke<CloudUpdateConfig>("get_cloud_update_config_command");
}

export async function saveCloudUpdateConfig(
  supabaseUrl: string,
  supabaseAnonKey: string
): Promise<CloudUpdateConfig> {
  return await invoke<CloudUpdateConfig>("save_cloud_update_config_command", {
    supabaseUrl,
    supabaseAnonKey,
  });
}

export async function checkForCloudUpdates(): Promise<CloudUpdateCheckResult> {
  return await invoke<CloudUpdateCheckResult>("check_for_cloud_updates_command");
}

export async function downloadAndInstallUpdate(
  version: string,
  downloadUrl: string,
  expectedSha256: string
): Promise<string> {
  return await invoke<string>("download_and_install_update_command", {
    version,
    downloadUrl,
    expectedSha256,
  });
}

export async function openExternalUrl(url: string): Promise<void> {
  await invoke<void>("open_external_url_command", { url });
}

export async function fetchStartupAndBoosterStatus(): Promise<StartupBoosterSnapshot> {
  return await invoke<StartupBoosterSnapshot>(
    "get_startup_and_booster_status_command"
  );
}

export async function removeStartupItem(
  name: string,
  commandOrPath: string,
  sourceType: string
): Promise<string> {
  return await invoke<string>("remove_startup_item_command", {
    name,
    commandOrPath,
    sourceType,
  });
}

export async function executeSystemBoostAction(
  actionId: string
): Promise<BoostActionOutcome> {
  return await invoke<BoostActionOutcome>(
    "execute_system_boost_action_command",
    { actionId }
  );
}

export async function fetchMemoryEngineStatus(): Promise<MemoryEngineStateResponse> {
  return await invoke<MemoryEngineStateResponse>(
    "get_memory_engine_status_command"
  );
}

export async function saveMemoryEngineConfig(
  cleanDepth: string,
  autoIntervalMinutes: number,
  autoThresholdPercent: number,
  cleanOnAppStartup: boolean,
  launchAppOnWindowsStartup: boolean,
  runInBackground: boolean,
  showNotifications: boolean
): Promise<MemoryEngineStateResponse> {
  return await invoke<MemoryEngineStateResponse>(
    "save_memory_engine_config_command",
    {
      cleanDepth,
      autoIntervalMinutes,
      autoThresholdPercent,
      cleanOnAppStartup,
      launchAppOnWindowsStartup,
      runInBackground,
      showNotifications,
    }
  );
}

export async function executeMemoryClean(
  depth: string,
  triggerSource?: string
): Promise<MemoryCleanReport> {
  return await invoke<MemoryCleanReport>("execute_memory_clean_command", {
    depth,
    triggerSource: triggerSource ?? "manual",
  });
}

export async function trimSingleProcess(
  pid: number,
  processName: string
): Promise<number> {
  return await invoke<number>("trim_single_process_command", {
    pid,
    processName,
  });
}

export function formatBytes(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes <= 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  const idx = Math.min(
    Math.floor(Math.log(bytes) / Math.log(1024)),
    units.length - 1
  );
  const value = bytes / Math.pow(1024, idx);
  return `${value.toFixed(idx === 0 ? 0 : 2)} ${units[idx]}`;
}

export function splitBytesParts(bytes: number): { value: string; unit: string } {
  if (!Number.isFinite(bytes) || bytes <= 0) {
    return { value: "0", unit: "B" };
  }
  const units = ["B", "KB", "MB", "GB", "TB"];
  const idx = Math.min(
    Math.floor(Math.log(bytes) / Math.log(1024)),
    units.length - 1
  );
  const value = bytes / Math.pow(1024, idx);
  return {
    value: value.toFixed(idx === 0 ? 0 : 1),
    unit: units[idx],
  };
}
