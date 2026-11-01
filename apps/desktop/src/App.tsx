import React, { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  Activity,
  AlertTriangle,
  ArrowUp,
  Check,
  Copy,
  Cpu,
  Database,
  ExternalLink,
  FileSearch,
  FolderOpen,
  FolderSearch,
  HardDrive,
  History,
  Layers,
  Moon,
  Package,
  Play,
  RefreshCw,
  Settings,
  ShieldAlert,
  ShieldCheck,
  Sparkles,
  Square,
  Sun,
  Trash2,
} from "lucide-react";
import { TRANSLATIONS } from "./i18n/translations";
import {
  cancelActiveJob,
  checkForCloudUpdates,
  downloadAndInstallUpdate,
  executeCleanupPlan,
  executeDuplicateRemoval,
  executeLeftoverRemoval,
  executeMemoryClean,
  executeSystemBoostAction,
  fetchAuditHistory,
  fetchCleanupPlanResult,
  fetchCleanupRules,
  fetchCloudUpdateConfig,
  fetchDiskScanResult,
  fetchDuplicateScanResult,
  fetchInventoryScanResult,
  fetchJobProgress,
  fetchLatestCleanupPlan,
  fetchLatestDiskScan,
  fetchLatestDuplicateScan,
  fetchLatestInventoryResult,
  fetchMemoryEngineStatus,
  fetchQuickScanPresets,
  fetchStartupAndBoosterStatus,
  fetchSystemFoundationStatus,
  formatBytes,
  launchAppUninstaller,
  openPathInExplorer,
  removeStartupItem,
  saveMemoryEngineConfig,
  splitBytesParts,
  startCleanupScan,
  startDiskScan,
  startDuplicateScan,
  startInventoryScan,
  trimSingleProcess,
  updateUiPreferences,
} from "./services/ipc";
import {
  AuditEventRecord,
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
  NavSectionId,
  QuickScanPreset,
  StartupBoosterSnapshot,
  SystemFoundationStatus,
  ThemeMode,
  UserErrorPayload,
} from "./types/saleem";

export function App() {
  const [activeNav, setActiveNav] = useState<NavSectionId>("overview");
  const [lang, setLang] = useState<LanguageCode>("ar");
  const [theme, setTheme] = useState<ThemeMode>("light");
  const [status, setStatus] = useState<SystemFoundationStatus | null>(null);
  const [presets, setPresets] = useState<QuickScanPreset[]>([]);
  const [auditEvents, setAuditEvents] = useState<AuditEventRecord[]>([]);
  const [, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<UserErrorPayload | null>(null);

  // Disk Analyzer State (Phase 1)
  const [targetPathInput, setTargetPathInput] = useState<string>("");
  const [activeJobId, setActiveJobId] = useState<string | null>(null);
  const [scanProgress, setScanProgress] = useState<JobProgressSnapshot | null>(null);
  const [scanResult, setScanResult] = useState<DiskScanResult | null>(null);
  const [mapMode, setMapMode] = useState<"bubbles" | "treemap">("bubbles");
  const [folderSearch, setFolderSearch] = useState<string>("");
  const [folderSort, setFolderSort] = useState<
    "logical" | "allocated" | "files" | "name"
  >("logical");
  const [largeFileSearch, setLargeFileSearch] = useState<string>("");
  const [selectedCategory, setSelectedCategory] = useState<string>("ALL");
  const [largeFileMinSize, setLargeFileMinSize] = useState<number>(0);
  const [selectedLargeFilePaths, setSelectedLargeFilePaths] = useState<string[]>(
    []
  );
  const [confirmingLargeDelete, setConfirmingLargeDelete] =
    useState<boolean>(false);
  const [deletingLargeFiles, setDeletingLargeFiles] = useState<boolean>(false);
  const [largeDeleteReport, setLargeDeleteReport] =
    useState<CleanupExecutionReport | null>(null);
  const [copiedPath, setCopiedPath] = useState<string | null>(null);

  // Duplicate Finder State (Phase 2)
  const [dupMinSizeBytes, setDupMinSizeBytes] = useState<number>(1024);
  const [activeDupJobId, setActiveDupJobId] = useState<string | null>(null);
  const [dupProgress, setDupProgress] = useState<JobProgressSnapshot | null>(null);
  const [dupResult, setDupResult] = useState<DuplicateScanResult | null>(null);
  const [retainedByGroup, setRetainedByGroup] = useState<Record<string, string>>({});
  const [selectedDeletesByGroup, setSelectedDeletesByGroup] = useState<
    Record<string, string[]>
  >({});
  const [confirmingDupRemoval, setConfirmingDupRemoval] = useState<boolean>(false);
  const [removingDuplicates, setRemovingDuplicates] = useState<boolean>(false);
  const [dupRemovalReport, setDupRemovalReport] =
    useState<DuplicateRemovalReport | null>(null);

  // Safe Cleanup Engine State (Phase 3)
  const [cleanupRules, setCleanupRules] = useState<CleanupRuleDefinition[]>([]);
  const [enabledRuleIds, setEnabledRuleIds] = useState<string[]>([]);
  const [activeCleanupJobId, setActiveCleanupJobId] = useState<string | null>(
    null
  );
  const [cleanupProgress, setCleanupProgress] =
    useState<JobProgressSnapshot | null>(null);
  const [cleanupPlan, setCleanupPlan] = useState<CleanupPlanResult | null>(null);
  const [selectedCleanupPaths, setSelectedCleanupPaths] = useState<string[]>([]);
  const [confirmingCleanup, setConfirmingCleanup] = useState<boolean>(false);
  const [executingCleanup, setExecutingCleanup] = useState<boolean>(false);
  const [cleanupReport, setCleanupReport] =
    useState<CleanupExecutionReport | null>(null);

  // App, Game & Leftover Inventory State (Phase 4)
  const [activeInventoryJobId, setActiveInventoryJobId] = useState<string | null>(
    null
  );
  const [inventoryProgress, setInventoryProgress] =
    useState<JobProgressSnapshot | null>(null);
  const [inventoryResult, setInventoryResult] =
    useState<InventoryScanResult | null>(null);
  const [inventoryTab, setInventoryTab] = useState<"leftovers" | "installed">(
    "leftovers"
  );
  const [leftoverFilter, setLeftoverFilter] = useState<
    "ALL" | "High" | "Medium" | "Low" | "SAVED"
  >("ALL");
  const [leftoverSearch, setLeftoverSearch] = useState<string>("");
  const [appSourceFilter, setAppSourceFilter] = useState<string>("ALL");
  const [appSearch, setAppSearch] = useState<string>("");
  const [selectedLeftoverPaths, setSelectedLeftoverPaths] = useState<string[]>(
    []
  );
  const [confirmingLeftoverRemoval, setConfirmingLeftoverRemoval] =
    useState<boolean>(false);
  const [removingLeftovers, setRemovingLeftovers] = useState<boolean>(false);
  const [leftoverRemovalReport, setLeftoverRemovalReport] =
    useState<LeftoverRemovalReport | null>(null);
  const [uninstallerNotice, setUninstallerNotice] = useState<string | null>(
    null
  );

  // Official Auto-Updater State
  const [cloudConfig, setCloudConfig] = useState<CloudUpdateConfig | null>(
    null
  );
  const [checkingUpdate, setCheckingUpdate] = useState<boolean>(false);
  const [downloadingUpdate, setDownloadingUpdate] = useState<boolean>(false);
  const [updateInstallDone, setUpdateInstallDone] = useState<boolean>(false);
  const [updateCheckResult, setUpdateCheckResult] =
    useState<CloudUpdateCheckResult | null>(null);
  const [startupSnapshot, setStartupSnapshot] =
    useState<StartupBoosterSnapshot | null>(null);
  const [boostNotice, setBoostNotice] = useState<string | null>(null);
  const [runningBoost, setRunningBoost] = useState<boolean>(false);

  // Standalone Memory Engine (SALEEM.MemoryEngine.dll — MemReduct Pro) State
  const [memoryState, setMemoryState] =
    useState<MemoryEngineStateResponse | null>(null);
  const [memoryReport, setMemoryReport] = useState<MemoryCleanReport | null>(
    null
  );
  const [cleaningMemory, setCleaningMemory] = useState<boolean>(false);
  const [trimmingPid, setTrimmingPid] = useState<number | null>(null);
  const [processTrimNotice, setProcessTrimNotice] = useState<string | null>(
    null
  );
  const [settingsTab, setSettingsTab] = useState<
    "updates" | "engines" | "policy"
  >("updates");
  const startupRamCleanedRef = useRef<boolean>(false);

  const pollTimerRef = useRef<number | null>(null);
  const dupPollTimerRef = useRef<number | null>(null);
  const cleanupPollTimerRef = useRef<number | null>(null);
  const inventoryPollTimerRef = useRef<number | null>(null);
  const t = TRANSLATIONS[lang];

  useEffect(() => {
    document.documentElement.lang = lang;
    document.documentElement.dir = lang === "ar" ? "rtl" : "ltr";
    document.documentElement.setAttribute("data-theme", theme);
  }, [lang, theme]);

  const initializeDuplicateSelections = useCallback(
    (res: DuplicateScanResult) => {
      const nextRetained: Record<string, string> = {};
      const nextSelected: Record<string, string[]> = {};
      for (const grp of res.groups) {
        if (grp.copies.length === 0) continue;
        const suggested =
          grp.copies.find((c) => c.suggested_keep) || grp.copies[0];
        nextRetained[grp.group_id] = suggested.path;
        nextSelected[grp.group_id] = [];
      }
      setRetainedByGroup(nextRetained);
      setSelectedDeletesByGroup(nextSelected);
      setConfirmingDupRemoval(false);
    },
    []
  );

  const loadBackendData = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const [
        sysStatus,
        history,
        quickPresets,
        latestScan,
        latestDup,
        rules,
        latestPlan,
        latestInv,
        cloudCfg,
        boosterSnap,
        memStatus,
      ] = await Promise.all([
        fetchSystemFoundationStatus(),
        fetchAuditHistory(50),
        fetchQuickScanPresets(),
        fetchLatestDiskScan(),
        fetchLatestDuplicateScan(),
        fetchCleanupRules(),
        fetchLatestCleanupPlan(),
        fetchLatestInventoryResult(),
        fetchCloudUpdateConfig(),
        fetchStartupAndBoosterStatus(),
        fetchMemoryEngineStatus(),
      ]);
      setStatus(sysStatus);
      setCloudConfig(cloudCfg);
      setStartupSnapshot(boosterSnap);
      setMemoryState(memStatus);
      if (memStatus.last_report && !memoryReport) {
        setMemoryReport(memStatus.last_report);
      }
      if (
        memStatus.config.clean_on_app_startup &&
        !startupRamCleanedRef.current
      ) {
        startupRamCleanedRef.current = true;
        void executeMemoryClean(
          memStatus.config.clean_depth,
          "app_startup"
        ).then((rep) => {
          setMemoryReport(rep);
          void fetchMemoryEngineStatus().then(setMemoryState);
        });
      }
      setAuditEvents(history);
      setPresets(quickPresets);
      setCleanupRules(rules);
      setEnabledRuleIds((prev) =>
        prev.length > 0
          ? prev
          : rules.filter((r) => r.enabled_by_default).map((r) => r.id)
      );
      if (latestPlan) {
        setCleanupPlan(latestPlan);
        setSelectedCleanupPaths(
          latestPlan.candidates
            .filter((c) => c.risk_level === "Low" && c.safe_to_clean)
            .map((c) => c.path)
        );
      }
      if (latestInv) {
        setInventoryResult(latestInv);
        setSelectedLeftoverPaths(
          latestInv.leftover_candidates
            .filter((c) => c.recommended_select)
            .map((c) => c.path)
        );
      }
      if (latestScan) {
        setScanResult(latestScan);
        setTargetPathInput(latestScan.target_path);
      } else if (quickPresets.length > 0 && !targetPathInput) {
        const downloadsPreset =
          quickPresets.find((p) => p.id === "downloads") || quickPresets[0];
        setTargetPathInput(downloadsPreset.path);
      }
      if (latestDup) {
        setDupResult(latestDup);
        initializeDuplicateSelections(latestDup);
      }
      if (sysStatus.ui_language === "ar" || sysStatus.ui_language === "en") {
        setLang(sysStatus.ui_language);
      }
      if (sysStatus.ui_theme === "dark" || sysStatus.ui_theme === "light") {
        setTheme(sysStatus.ui_theme);
      }
    } catch (err: unknown) {
      const fallback: UserErrorPayload =
        typeof err === "object" && err !== null && "error_code" in err
          ? (err as UserErrorPayload)
          : {
              error_code: "IPC_CONNECTION_ERROR",
              message_en: "Could not communicate with the SALEEM Rust backend.",
              message_ar: "تعذر الاتصال بمحرك SALEEM الخلفي (Rust IPC).",
              technical_hint: String(err),
            };
      setError(fallback);
    } finally {
      setLoading(false);
    }
  }, [targetPathInput]);

  useEffect(() => {
    void loadBackendData();
    return () => {
      if (pollTimerRef.current) {
        window.clearInterval(pollTimerRef.current);
      }
      if (dupPollTimerRef.current) {
        window.clearInterval(dupPollTimerRef.current);
      }
      if (cleanupPollTimerRef.current) {
        window.clearInterval(cleanupPollTimerRef.current);
      }
    };
  }, []);

  const triggerScanOnPath = useCallback(
    async (pathToScan: string, switchView = false) => {
      const trimmed = pathToScan.trim();
      if (!trimmed) return;
      setError(null);
      setTargetPathInput(trimmed);
      if (switchView) {
        setActiveNav("storage");
      }

      try {
        const jobId = await startDiskScan(trimmed, 120, 80);
        setActiveJobId(jobId);

        if (pollTimerRef.current) {
          window.clearInterval(pollTimerRef.current);
        }

        pollTimerRef.current = window.setInterval(async () => {
          try {
            const snap = await fetchJobProgress(jobId);
            setScanProgress(snap);
            if (snap.status === "Completed") {
              if (pollTimerRef.current) window.clearInterval(pollTimerRef.current);
              pollTimerRef.current = null;
              setActiveJobId(null);
              const res = await fetchDiskScanResult(jobId);
              if (res) {
                setScanResult(res);
              }
              const [sys, hist] = await Promise.all([
                fetchSystemFoundationStatus(),
                fetchAuditHistory(50),
              ]);
              setStatus(sys);
              setAuditEvents(hist);
            } else if (snap.status === "Cancelled" || snap.status === "Failed") {
              if (pollTimerRef.current) window.clearInterval(pollTimerRef.current);
              pollTimerRef.current = null;
              setActiveJobId(null);
              const hist = await fetchAuditHistory(50);
              setAuditEvents(hist);
            }
          } catch {
            if (pollTimerRef.current) window.clearInterval(pollTimerRef.current);
            pollTimerRef.current = null;
            setActiveJobId(null);
          }
        }, 180);
      } catch (err: unknown) {
        const payload: UserErrorPayload =
          typeof err === "object" && err !== null && "error_code" in err
            ? (err as UserErrorPayload)
            : {
                error_code: "SCAN_START_FAILED",
                message_en: "Failed to start directory scan.",
                message_ar: "تعذر بدء فحص المسار المحدد.",
                technical_hint: String(err),
              };
        setError(payload);
      }
    },
    []
  );

  const handleCancelScan = async () => {
    if (!activeJobId) return;
    try {
      await cancelActiveJob(activeJobId);
    } catch {
      // ignore
    }
  };

  // Phase 2: Duplicate Scan & Safe Selection Handlers
  const triggerDuplicateScan = useCallback(
    async (pathToScan: string) => {
      const trimmed = pathToScan.trim();
      if (!trimmed) return;
      setError(null);
      setDupRemovalReport(null);
      setConfirmingDupRemoval(false);
      setTargetPathInput(trimmed);

      try {
        const jobId = await startDuplicateScan(trimmed, dupMinSizeBytes, 150);
        setActiveDupJobId(jobId);

        if (dupPollTimerRef.current) {
          window.clearInterval(dupPollTimerRef.current);
        }

        dupPollTimerRef.current = window.setInterval(async () => {
          try {
            const snap = await fetchJobProgress(jobId);
            setDupProgress(snap);
            if (snap.status === "Completed") {
              if (dupPollTimerRef.current)
                window.clearInterval(dupPollTimerRef.current);
              dupPollTimerRef.current = null;
              setActiveDupJobId(null);
              const res = await fetchDuplicateScanResult(jobId);
              if (res) {
                setDupResult(res);
                initializeDuplicateSelections(res);
              }
              const [sys, hist] = await Promise.all([
                fetchSystemFoundationStatus(),
                fetchAuditHistory(50),
              ]);
              setStatus(sys);
              setAuditEvents(hist);
            } else if (snap.status === "Cancelled" || snap.status === "Failed") {
              if (dupPollTimerRef.current)
                window.clearInterval(dupPollTimerRef.current);
              dupPollTimerRef.current = null;
              setActiveDupJobId(null);
              const hist = await fetchAuditHistory(50);
              setAuditEvents(hist);
            }
          } catch {
            if (dupPollTimerRef.current)
              window.clearInterval(dupPollTimerRef.current);
            dupPollTimerRef.current = null;
            setActiveDupJobId(null);
          }
        }, 180);
      } catch (err: unknown) {
        const payload: UserErrorPayload =
          typeof err === "object" && err !== null && "error_code" in err
            ? (err as UserErrorPayload)
            : {
                error_code: "DUPLICATE_SCAN_FAILED",
                message_en: "Failed to start duplicate scan.",
                message_ar: "تعذر بدء فحص الملفات المكررة.",
                technical_hint: String(err),
              };
        setError(payload);
      }
    },
    [dupMinSizeBytes, initializeDuplicateSelections]
  );

  const handleCancelDupScan = async () => {
    if (!activeDupJobId) return;
    try {
      await cancelActiveJob(activeDupJobId);
    } catch {
      // ignore
    }
  };

  const handleSetRetainedCopy = (groupId: string, retainedPath: string) => {
    setRetainedByGroup((prev) => ({
      ...prev,
      [groupId]: retainedPath,
    }));
    // Ensure the retained copy is never in the delete list
    setSelectedDeletesByGroup((prev) => ({
      ...prev,
      [groupId]: (prev[groupId] || []).filter((p) => p !== retainedPath),
    }));
    setConfirmingDupRemoval(false);
  };

  const handleToggleDeleteMember = (groupId: string, memberPath: string) => {
    const currentRetained = retainedByGroup[groupId];
    if (currentRetained === memberPath) {
      // Never allow selecting the retained copy for deletion
      return;
    }
    setSelectedDeletesByGroup((prev) => {
      const current = prev[groupId] || [];
      const exists = current.includes(memberPath);
      return {
        ...prev,
        [groupId]: exists
          ? current.filter((p) => p !== memberPath)
          : [...current, memberPath],
      };
    });
    setConfirmingDupRemoval(false);
  };

  const handleSmartSelectDuplicates = (mode: "keep_oldest" | "keep_newest") => {
    if (!dupResult) return;
    const nextRetained: Record<string, string> = {};
    const nextSelected: Record<string, string[]> = {};

    for (const grp of dupResult.groups) {
      if (grp.copies.length < 2) continue;
      const sorted = [...grp.copies].sort((a, b) =>
        mode === "keep_oldest"
          ? a.modified_at.localeCompare(b.modified_at)
          : b.modified_at.localeCompare(a.modified_at)
      );
      const keep = sorted[0];
      nextRetained[grp.group_id] = keep.path;
      // Select all non-hardlink-alias copies except the retained one
      nextSelected[grp.group_id] = sorted
        .slice(1)
        .filter((c) => !c.is_hardlink_alias)
        .map((c) => c.path);
    }

    setRetainedByGroup(nextRetained);
    setSelectedDeletesByGroup(nextSelected);
    setConfirmingDupRemoval(false);
  };

  const handleClearDuplicateSelection = () => {
    if (!dupResult) return;
    const cleared: Record<string, string[]> = {};
    for (const grp of dupResult.groups) {
      cleared[grp.group_id] = [];
    }
    setSelectedDeletesByGroup(cleared);
    setConfirmingDupRemoval(false);
  };

  const duplicateSelectionSummary = useMemo(() => {
    if (!dupResult) {
      return {
        totalFilesSelected: 0,
        totalBytesSelected: 0,
        selections: [] as DuplicateRemovalGroupSelection[],
      };
    }
    let totalFilesSelected = 0;
    let totalBytesSelected = 0;
    const selections: DuplicateRemovalGroupSelection[] = [];

    for (const grp of dupResult.groups) {
      const retainedPath =
        retainedByGroup[grp.group_id] || grp.copies[0]?.path || "";
      const delPaths = (selectedDeletesByGroup[grp.group_id] || []).filter(
        (p) => p !== retainedPath
      );
      if (delPaths.length > 0 && retainedPath) {
        totalFilesSelected += delPaths.length;
        totalBytesSelected += delPaths.length * grp.file_size_bytes;
        selections.push({
          group_sha256: grp.sha256,
          file_size_bytes: grp.file_size_bytes,
          retained_reference_path: retainedPath,
          delete_paths: delPaths,
        });
      }
    }

    return { totalFilesSelected, totalBytesSelected, selections };
  }, [dupResult, retainedByGroup, selectedDeletesByGroup]);

  const handleExecuteDuplicateRemoval = async () => {
    if (duplicateSelectionSummary.selections.length === 0) return;
    setRemovingDuplicates(true);
    setError(null);
    try {
      const report = await executeDuplicateRemoval(
        duplicateSelectionSummary.selections
      );
      setDupRemovalReport(report);
      setConfirmingDupRemoval(false);
      const [latestDup, sys, hist] = await Promise.all([
        fetchLatestDuplicateScan(),
        fetchSystemFoundationStatus(),
        fetchAuditHistory(50),
      ]);
      if (latestDup) {
        setDupResult(latestDup);
        initializeDuplicateSelections(latestDup);
      }
      setStatus(sys);
      setAuditEvents(hist);
    } catch (err: unknown) {
      const payload: UserErrorPayload =
        typeof err === "object" && err !== null && "error_code" in err
          ? (err as UserErrorPayload)
          : {
              error_code: "DUPLICATE_REMOVAL_FAILED",
              message_en: "Failed to execute verified duplicate removal.",
              message_ar: "تعذر تنفيذ حذف النسخ المكررة المحددة.",
              technical_hint: String(err),
            };
      setError(payload);
    } finally {
      setRemovingDuplicates(false);
    }
  };

  // Phase 3: Safe Cleanup Engine & User-Controlled Large File Removal Handlers
  const handleToggleRuleEnabled = (ruleId: string) => {
    setEnabledRuleIds((prev) =>
      prev.includes(ruleId)
        ? prev.filter((id) => id !== ruleId)
        : [...prev, ruleId]
    );
  };

  const triggerSafeCleanupScan = useCallback(async () => {
    setError(null);
    setCleanupReport(null);
    setConfirmingCleanup(false);
    try {
      const jobId = await startCleanupScan(
        enabledRuleIds.length > 0 ? enabledRuleIds : undefined
      );
      setActiveCleanupJobId(jobId);

      if (cleanupPollTimerRef.current) {
        window.clearInterval(cleanupPollTimerRef.current);
      }

      cleanupPollTimerRef.current = window.setInterval(async () => {
        try {
          const snap = await fetchJobProgress(jobId);
          setCleanupProgress(snap);
          if (snap.status === "Completed") {
            if (cleanupPollTimerRef.current)
              window.clearInterval(cleanupPollTimerRef.current);
            cleanupPollTimerRef.current = null;
            setActiveCleanupJobId(null);
            const plan = await fetchCleanupPlanResult(jobId);
            if (plan) {
              setCleanupPlan(plan);
              setSelectedCleanupPaths(
                plan.candidates
                  .filter((c) => c.risk_level === "Low" && c.safe_to_clean)
                  .map((c) => c.path)
              );
            }
            const [sys, hist] = await Promise.all([
              fetchSystemFoundationStatus(),
              fetchAuditHistory(50),
            ]);
            setStatus(sys);
            setAuditEvents(hist);
          } else if (snap.status === "Cancelled" || snap.status === "Failed") {
            if (cleanupPollTimerRef.current)
              window.clearInterval(cleanupPollTimerRef.current);
            cleanupPollTimerRef.current = null;
            setActiveCleanupJobId(null);
          }
        } catch {
          if (cleanupPollTimerRef.current)
            window.clearInterval(cleanupPollTimerRef.current);
          cleanupPollTimerRef.current = null;
          setActiveCleanupJobId(null);
        }
      }, 180);
    } catch (err: unknown) {
      const payload: UserErrorPayload =
        typeof err === "object" && err !== null && "error_code" in err
          ? (err as UserErrorPayload)
          : {
              error_code: "CLEANUP_SCAN_FAILED",
              message_en: "Failed to start safe cleanup scan.",
              message_ar: "تعذر بدء فحص التنظيف الآمن.",
              technical_hint: String(err),
            };
      setError(payload);
    }
  }, [enabledRuleIds]);

  const handleCancelCleanupScan = async () => {
    if (!activeCleanupJobId) return;
    try {
      await cancelActiveJob(activeCleanupJobId);
    } catch {
      // ignore
    }
  };

  const handleToggleCleanupCandidate = (path: string) => {
    setSelectedCleanupPaths((prev) =>
      prev.includes(path) ? prev.filter((p) => p !== path) : [...prev, path]
    );
    setConfirmingCleanup(false);
  };

  const handleSelectAllSafeCleanup = () => {
    if (!cleanupPlan) return;
    setSelectedCleanupPaths(
      cleanupPlan.candidates
        .filter((c) => c.risk_level === "Low" && c.safe_to_clean)
        .map((c) => c.path)
    );
    setConfirmingCleanup(false);
  };

  const cleanupSelectionSummary = useMemo(() => {
    if (!cleanupPlan) {
      return {
        count: 0,
        bytes: 0,
        items: [] as CleanupExecutionItemRequest[],
      };
    }
    const items: CleanupExecutionItemRequest[] = [];
    let bytes = 0;
    for (const c of cleanupPlan.candidates) {
      if (selectedCleanupPaths.includes(c.path)) {
        bytes += c.logical_size;
        items.push({
          path: c.path,
          rule_id: c.rule_id,
          expected_size_bytes: c.logical_size,
        });
      }
    }
    return { count: items.length, bytes, items };
  }, [cleanupPlan, selectedCleanupPaths]);

  const handleExecuteSafeCleanup = async () => {
    if (!cleanupPlan || cleanupSelectionSummary.items.length === 0) return;
    setExecutingCleanup(true);
    setError(null);
    try {
      const report = await executeCleanupPlan(
        cleanupPlan.plan_id,
        cleanupSelectionSummary.items
      );
      setCleanupReport(report);
      setConfirmingCleanup(false);
      const [latestPlan, sys, hist] = await Promise.all([
        fetchLatestCleanupPlan(),
        fetchSystemFoundationStatus(),
        fetchAuditHistory(50),
      ]);
      if (latestPlan) {
        setCleanupPlan(latestPlan);
        setSelectedCleanupPaths([]);
      }
      setStatus(sys);
      setAuditEvents(hist);
    } catch (err: unknown) {
      const payload: UserErrorPayload =
        typeof err === "object" && err !== null && "error_code" in err
          ? (err as UserErrorPayload)
          : {
              error_code: "CLEANUP_EXEC_FAILED",
              message_en: "Failed to execute safe cleanup plan.",
              message_ar: "تعذر تنفيذ خطة التنظيف الآمن.",
              technical_hint: String(err),
            };
      setError(payload);
    } finally {
      setExecutingCleanup(false);
    }
  };

  // User-Controlled Large File Selection & Verified Removal
  const handleToggleLargeFileSelect = (path: string) => {
    setSelectedLargeFilePaths((prev) =>
      prev.includes(path) ? prev.filter((p) => p !== path) : [...prev, path]
    );
    setConfirmingLargeDelete(false);
  };

  const largeFileSelectionSummary = useMemo(() => {
    if (!scanResult) {
      return {
        count: 0,
        bytes: 0,
        items: [] as CleanupExecutionItemRequest[],
      };
    }
    const items: CleanupExecutionItemRequest[] = [];
    let bytes = 0;
    for (const f of scanResult.top_files) {
      if (selectedLargeFilePaths.includes(f.path)) {
        bytes += f.logical_size;
        items.push({
          path: f.path,
          rule_id: "USER-LARGE-FILE-SELECT",
          expected_size_bytes: f.logical_size,
        });
      }
    }
    return { count: items.length, bytes, items };
  }, [scanResult, selectedLargeFilePaths]);

  const handleExecuteLargeFileDelete = async () => {
    if (!scanResult || largeFileSelectionSummary.items.length === 0) return;
    setDeletingLargeFiles(true);
    setError(null);
    try {
      const report = await executeCleanupPlan(
        `large-files-${scanResult.scan_id}`,
        largeFileSelectionSummary.items
      );
      setLargeDeleteReport(report);
      setConfirmingLargeDelete(false);
      setSelectedLargeFilePaths([]);
      const [latestDisk, sys, hist] = await Promise.all([
        fetchLatestDiskScan(),
        fetchSystemFoundationStatus(),
        fetchAuditHistory(50),
      ]);
      if (latestDisk) {
        setScanResult(latestDisk);
      }
      setStatus(sys);
      setAuditEvents(hist);
    } catch (err: unknown) {
      const payload: UserErrorPayload =
        typeof err === "object" && err !== null && "error_code" in err
          ? (err as UserErrorPayload)
          : {
              error_code: "LARGE_FILE_DELETE_FAILED",
              message_en: "Failed to remove selected large files.",
              message_ar: "تعذر حذف الملفات الضخمة المحددة.",
              technical_hint: String(err),
            };
      setError(payload);
    } finally {
      setDeletingLargeFiles(false);
    }
  };

  // Phase 4: Application, Game & Leftover Inventory Handlers
  const handleStartInventoryScan = useCallback(async () => {
    setError(null);
    setLeftoverRemovalReport(null);
    try {
      const jobId = await startInventoryScan();
      setActiveInventoryJobId(jobId);

      if (inventoryPollTimerRef.current) {
        window.clearInterval(inventoryPollTimerRef.current);
      }

      inventoryPollTimerRef.current = window.setInterval(async () => {
        try {
          const prog = await fetchJobProgress(jobId);
          if (prog) {
            setInventoryProgress(prog);
            if (
              prog.status === "Completed" ||
              prog.status === "Cancelled" ||
              prog.status === "Failed"
            ) {
              if (inventoryPollTimerRef.current) {
                window.clearInterval(inventoryPollTimerRef.current);
                inventoryPollTimerRef.current = null;
              }
              setActiveInventoryJobId(null);

              const [res, sys, hist] = await Promise.all([
                fetchInventoryScanResult(jobId),
                fetchSystemFoundationStatus(),
                fetchAuditHistory(50),
              ]);
              if (res) {
                setInventoryResult(res);
                setSelectedLeftoverPaths(
                  res.leftover_candidates
                    .filter((c) => c.recommended_select)
                    .map((c) => c.path)
                );
              }
              setStatus(sys);
              setAuditEvents(hist);
            }
          }
        } catch {
          // ignore transient poll error
        }
      }, 220);
    } catch (err: unknown) {
      const payload: UserErrorPayload =
        typeof err === "object" && err !== null && "error_code" in err
          ? (err as UserErrorPayload)
          : {
              error_code: "INVENTORY_SCAN_FAILED",
              message_en: "Failed to start application & leftover inventory scan.",
              message_ar: "تعذر بدء فحص البرامج والبقايا اليتيمة.",
              technical_hint: String(err),
            };
      setError(payload);
    }
  }, []);

  const handleCancelInventoryScan = async () => {
    if (!activeInventoryJobId) return;
    try {
      await cancelActiveJob(activeInventoryJobId);
    } catch {
      // ignore
    }
  };

  const handleToggleLeftoverSelect = (path: string) => {
    setSelectedLeftoverPaths((prev) =>
      prev.includes(path) ? prev.filter((p) => p !== path) : [...prev, path]
    );
    setConfirmingLeftoverRemoval(false);
  };

  const handleSelectHighConfidenceLeftovers = () => {
    if (!inventoryResult) return;
    setSelectedLeftoverPaths(
      inventoryResult.leftover_candidates
        .filter((c) => c.recommended_select)
        .map((c) => c.path)
    );
    setConfirmingLeftoverRemoval(false);
  };

  const leftoverSelectionSummary = useMemo(() => {
    if (!inventoryResult) {
      return {
        count: 0,
        bytes: 0,
        items: [] as LeftoverRemovalItemRequest[],
      };
    }
    const items: LeftoverRemovalItemRequest[] = [];
    let bytes = 0;
    for (const c of inventoryResult.leftover_candidates) {
      if (selectedLeftoverPaths.includes(c.path)) {
        bytes += c.logical_size_bytes;
        items.push({
          path: c.path,
          expected_file_count: c.file_count,
          expected_logical_bytes: c.logical_size_bytes,
          confidence: c.confidence,
        });
      }
    }
    return { count: items.length, bytes, items };
  }, [inventoryResult, selectedLeftoverPaths]);

  const handleExecuteLeftoverRemoval = async () => {
    if (!inventoryResult || leftoverSelectionSummary.items.length === 0) return;
    setRemovingLeftovers(true);
    setError(null);
    try {
      const report = await executeLeftoverRemoval(
        inventoryResult.scan_id,
        leftoverSelectionSummary.items
      );
      setLeftoverRemovalReport(report);
      setConfirmingLeftoverRemoval(false);
      setSelectedLeftoverPaths([]);
      const [latestInv, sys, hist] = await Promise.all([
        fetchLatestInventoryResult(),
        fetchSystemFoundationStatus(),
        fetchAuditHistory(50),
      ]);
      if (latestInv) {
        setInventoryResult(latestInv);
      }
      setStatus(sys);
      setAuditEvents(hist);
    } catch (err: unknown) {
      const payload: UserErrorPayload =
        typeof err === "object" && err !== null && "error_code" in err
          ? (err as UserErrorPayload)
          : {
              error_code: "LEFTOVER_REMOVE_FAILED",
              message_en: "Failed to remove selected leftover folders.",
              message_ar: "تعذر حذف مجلدات البقايا المحددة.",
              technical_hint: String(err),
            };
      setError(payload);
    } finally {
      setRemovingLeftovers(false);
    }
  };

  const handleLaunchAppUninstaller = async (
    appName: string,
    source: string,
    uninstallString: string | null
  ) => {
    setError(null);
    setUninstallerNotice(null);
    try {
      await launchAppUninstaller(appName, source, uninstallString);
      setUninstallerNotice(
        lang === "ar"
          ? `تم فتح أداة إلغاء التثبيت الرسمية لـ «${appName}». بعد انتهاء الإزالة، اضغط على «فحص البرامج والبقايا اليتيمة» لاكتشاف وتنظيف أي بقايا خلفها البرنامج.`
          : `Launched official uninstaller for "${appName}". Once removal completes, click "Scan Apps & Orphaned Leftovers" to clean up any remaining leftover folders.`
      );
      const hist = await fetchAuditHistory(50);
      setAuditEvents(hist);
    } catch (err: unknown) {
      const payload: UserErrorPayload =
        typeof err === "object" && err !== null && "error_code" in err
          ? (err as UserErrorPayload)
          : {
              error_code: "UNINSTALL_LAUNCH_FAILED",
              message_en: "Could not launch the application uninstaller.",
              message_ar: "تعذر تشغيل أداة إلغاء تثبيت التطبيق.",
              technical_hint: String(err),
            };
      setError(payload);
    }
  };

  const handleDownloadAndInstallUpdate = async (
    version: string,
    downloadUrl: string,
    expectedSha256: string
  ) => {
    setDownloadingUpdate(true);
    setUpdateInstallDone(false);
    setError(null);
    try {
      await downloadAndInstallUpdate(version, downloadUrl, expectedSha256);
      setUpdateInstallDone(true);
      const hist = await fetchAuditHistory(50);
      setAuditEvents(hist);
    } catch (err: unknown) {
      const payload: UserErrorPayload =
        typeof err === "object" && err !== null && "error_code" in err
          ? (err as UserErrorPayload)
          : {
              error_code: "CLOUD_UPDATE_INSTALL_FAILED",
              message_en: "Failed to download and install update in background.",
              message_ar: "تعذر تحميل وتثبيت التحديث في الخلفية.",
              technical_hint: String(err),
            };
      setError(payload);
    } finally {
      setDownloadingUpdate(false);
    }
  };

  const handleExecuteMemoryReduction = useCallback(
    async (customDepth?: string, sourceLabel?: string) => {
      const depthToUse =
        customDepth || memoryState?.config.clean_depth || "Deep";
      setCleaningMemory(true);
      setError(null);
      try {
        const rep = await executeMemoryClean(
          depthToUse,
          sourceLabel || "manual"
        );
        setMemoryReport(rep);
        const [memStatus, boosterSnap, hist] = await Promise.all([
          fetchMemoryEngineStatus(),
          fetchStartupAndBoosterStatus(),
          fetchAuditHistory(50),
        ]);
        setMemoryState(memStatus);
        setStartupSnapshot(boosterSnap);
        setAuditEvents(hist);
      } catch (err: unknown) {
        const payload: UserErrorPayload =
          typeof err === "object" && err !== null && "error_code" in err
            ? (err as UserErrorPayload)
            : {
                error_code: "MEMORY_CLEAN_FAILED",
                message_en: "Failed to execute SALEEM.MemoryEngine reduction.",
                message_ar: "تعذر تنفيذ تحرير الذاكرة عبر محرك SALEEM.MemoryEngine.dll.",
                technical_hint: String(err),
              };
        setError(payload);
      } finally {
        setCleaningMemory(false);
      }
    },
    [memoryState?.config.clean_depth]
  );

  const handleUpdateMemoryConfig = async (
    overrides: Partial<{
      clean_depth: string;
      auto_interval_minutes: number;
      auto_threshold_percent: number;
      clean_on_app_startup: boolean;
      launch_app_on_windows_startup: boolean;
      run_in_background: boolean;
      show_notifications: boolean;
    }>
  ) => {
    if (!memoryState) return;
    const cfg = memoryState.config;
    try {
      const updated = await saveMemoryEngineConfig(
        overrides.clean_depth ?? cfg.clean_depth,
        overrides.auto_interval_minutes ?? cfg.auto_interval_minutes,
        overrides.auto_threshold_percent ?? cfg.auto_threshold_percent,
        overrides.clean_on_app_startup ?? cfg.clean_on_app_startup,
        overrides.launch_app_on_windows_startup ??
          cfg.launch_app_on_windows_startup,
        overrides.run_in_background ?? cfg.run_in_background,
        overrides.show_notifications ?? cfg.show_notifications
      );
      setMemoryState(updated);
      const [boosterSnap, hist] = await Promise.all([
        fetchStartupAndBoosterStatus(),
        fetchAuditHistory(50),
      ]);
      setStartupSnapshot(boosterSnap);
      setAuditEvents(hist);
    } catch (err: unknown) {
      const payload: UserErrorPayload =
        typeof err === "object" && err !== null && "error_code" in err
          ? (err as UserErrorPayload)
          : {
              error_code: "MEMORY_CONFIG_SAVE_FAILED",
              message_en: "Could not save Memory Engine settings.",
              message_ar: "تعذر حفظ إعدادات محرك تنظيف الرامات.",
              technical_hint: String(err),
            };
      setError(payload);
    }
  };

  useEffect(() => {
    if (activeNav !== "memory" && activeNav !== "startup") return;
    const timer = window.setInterval(() => {
      void fetchMemoryEngineStatus().then((st) => {
        setMemoryState(st);
        if (st.last_report) {
          setMemoryReport(st.last_report);
        }
      });
    }, 5000);
    return () => window.clearInterval(timer);
  }, [activeNav]);

  const handleCheckCloudUpdate = async () => {
    setCheckingUpdate(true);
    setError(null);
    setUpdateInstallDone(false);
    setUpdateCheckResult(null);
    try {
      const res = await checkForCloudUpdates();
      setUpdateCheckResult(res);
      const hist = await fetchAuditHistory(50);
      setAuditEvents(hist);
      // Do NOT auto-install! User reviews release features, type, and date first, then clicks install if desired.
    } catch (err: unknown) {
      const payload: UserErrorPayload =
        typeof err === "object" && err !== null && "error_code" in err
          ? (err as UserErrorPayload)
          : {
              error_code: "CLOUD_UPDATE_CHECK_FAILED",
              message_en: "Could not check for official updates.",
              message_ar: "تعذر الاتصال بخادم التحديثات الرسمي.",
              technical_hint: String(err),
            };
      setError(payload);
    } finally {
      setCheckingUpdate(false);
    }
  };

  const filteredLeftovers = useMemo(() => {
    if (!inventoryResult) return [];
    const q = leftoverSearch.trim().toLowerCase();
    return inventoryResult.leftover_candidates.filter((c) => {
      if (leftoverFilter === "SAVED") {
        if (!c.contains_saved_games_or_user_data) return false;
      } else if (leftoverFilter !== "ALL") {
        if (c.confidence !== leftoverFilter) return false;
      }
      if (!q) return true;
      return (
        c.folder_name.toLowerCase().includes(q) ||
        c.path.toLowerCase().includes(q) ||
        c.scope_label.toLowerCase().includes(q)
      );
    });
  }, [inventoryResult, leftoverFilter, leftoverSearch]);

  const filteredInstalledApps = useMemo(() => {
    if (!inventoryResult) return [];
    const q = appSearch.trim().toLowerCase();
    return inventoryResult.installed_apps.filter((app) => {
      if (appSourceFilter !== "ALL" && app.source !== appSourceFilter) {
        return false;
      }
      if (!q) return true;
      return (
        app.display_name.toLowerCase().includes(q) ||
        app.publisher.toLowerCase().includes(q) ||
        (app.install_location && app.install_location.toLowerCase().includes(q))
      );
    });
  }, [inventoryResult, appSourceFilter, appSearch]);

  const handleCopyPath = async (p: string) => {
    try {
      await navigator.clipboard.writeText(p);
      setCopiedPath(p);
      setTimeout(() => setCopiedPath(null), 1600);
    } catch {
      // ignore
    }
  };

  const handleNavigateParent = () => {
    if (!scanResult) return;
    const current = scanResult.target_path.replace(/\\+$/, "");
    const lastSep = current.lastIndexOf("\\");
    if (lastSep >= 2) {
      const parent =
        lastSep === 2 ? `${current.slice(0, 2)}\\` : current.slice(0, lastSep);
      if (parent !== scanResult.target_path) {
        void triggerScanOnPath(parent, false);
      }
    }
  };

  const handleToggleLanguage = async () => {
    const nextLang: LanguageCode = lang === "ar" ? "en" : "ar";
    setLang(nextLang);
    try {
      await updateUiPreferences(nextLang, theme);
      const history = await fetchAuditHistory(50);
      setAuditEvents(history);
    } catch {
      // Non-fatal
    }
  };

  const handleToggleTheme = async () => {
    const nextTheme: ThemeMode = theme === "dark" ? "light" : "dark";
    setTheme(nextTheme);
    try {
      await updateUiPreferences(lang, nextTheme);
      const history = await fetchAuditHistory(50);
      setAuditEvents(history);
    } catch {
      // Non-fatal
    }
  };

  // Hero Display Metrics (100% Real Data)
  const totalMountedUsedBytes = useMemo(() => {
    if (!status) return 0;
    return status.volumes.reduce((acc, v) => acc + v.used_bytes, 0);
  }, [status]);

  const heroPrimaryBytes = scanResult
    ? scanResult.total_logical_bytes
    : totalMountedUsedBytes;
  const heroParts = splitBytesParts(heroPrimaryBytes);

  // Filtered & Sorted Folders in Storage Analyzer
  const displayedFolders = useMemo(() => {
    if (!scanResult) return [];
    const q = folderSearch.trim().toLowerCase();
    const list = scanResult.top_folders.filter(
      (f) =>
        !q ||
        f.folder_name.toLowerCase().includes(q) ||
        f.path.toLowerCase().includes(q)
    );
    return [...list].sort((a, b) => {
      if (folderSort === "allocated") return b.allocated_size - a.allocated_size;
      if (folderSort === "files") return b.files_count - a.files_count;
      if (folderSort === "name") return a.folder_name.localeCompare(b.folder_name);
      return b.logical_size - a.logical_size;
    });
  }, [scanResult, folderSearch, folderSort]);

  // Filtered Large Files
  const displayedLargeFiles = useMemo(() => {
    if (!scanResult) return [];
    const q = largeFileSearch.trim().toLowerCase();
    return scanResult.top_files.filter((file) => {
      if (file.logical_size < largeFileMinSize) {
        return false;
      }
      if (selectedCategory !== "ALL" && file.category !== selectedCategory) {
        return false;
      }
      if (!q) return true;
      return (
        file.file_name.toLowerCase().includes(q) ||
        file.extension.toLowerCase().includes(q) ||
        file.path.toLowerCase().includes(q)
      );
    });
  }, [scanResult, largeFileSearch, selectedCategory, largeFileMinSize]);

  // Spheres for Space Map Preview (either scanned immediate children or mounted volumes)
  const previewSpheres = useMemo(() => {
    if (scanResult && scanResult.immediate_children.length > 0) {
      const top = scanResult.immediate_children.slice(0, 6);
      const maxSz = Math.max(...top.map((i) => i.logical_size), 1);
      return top.map((item, idx) => {
        const ratio = Math.sqrt(item.logical_size / maxSz);
        const diameter = Math.round(68 + ratio * 54);
        return {
          key: item.path,
          name: item.folder_name,
          sizeText: formatBytes(item.logical_size),
          path: item.path,
          diameter,
          delay: `${idx * -0.8}s`,
        };
      });
    }
    if (status && status.volumes.length > 0) {
      const maxSz = Math.max(...status.volumes.map((v) => v.used_bytes), 1);
      return status.volumes.map((v, idx) => {
        const ratio = Math.sqrt(v.used_bytes / maxSz);
        const diameter = Math.round(78 + ratio * 48);
        return {
          key: v.mount_point,
          name: v.volume_label
            ? `${v.mount_point} (${v.volume_label})`
            : v.mount_point,
          sizeText: formatBytes(v.used_bytes),
          path: v.mount_point,
          diameter,
          delay: `${idx * -1.1}s`,
        };
      });
    }
    return [];
  }, [scanResult, status]);

  const mainGroupNav: { id: NavSectionId; icon: React.ReactNode; badge?: string }[] = [
    { id: "overview", icon: <Activity size={17} /> },
    {
      id: "storage",
      icon: <HardDrive size={17} />,
      badge: scanResult ? formatBytes(scanResult.total_logical_bytes) : undefined,
    },
    {
      id: "large_files",
      icon: <FolderSearch size={17} />,
      badge: scanResult ? `${scanResult.top_files.length}` : undefined,
    },
    {
      id: "cleanup",
      icon: <Sparkles size={17} />,
      badge: cleanupPlan ? `${cleanupPlan.total_candidates}` : undefined,
    },
    {
      id: "startup",
      icon: <Play size={17} />,
      badge: startupSnapshot ? `${startupSnapshot.startup_items.length}` : undefined,
    },
    {
      id: "memory",
      icon: <Cpu size={17} />,
      badge: memoryState
        ? `${memoryState.snapshot.memory_load_percent}%`
        : "DLL",
    },
    { id: "sentinel", icon: <ShieldCheck size={17} /> },
  ];

  const secondaryGroupNav: { id: NavSectionId; icon: React.ReactNode; badge?: string }[] = [
    {
      id: "duplicates",
      icon: <Layers size={17} />,
      badge: dupResult ? `${dupResult.total_duplicate_groups}` : undefined,
    },
    { id: "applications", icon: <Package size={17} /> },
    { id: "quarantine", icon: <ShieldAlert size={17} /> },
    { id: "history", icon: <History size={17} /> },
  ];

  return (
    <div className="ambient-canvas">
      <div className="ambient-blob blob-1" />
      <div className="ambient-blob blob-2" />
      <div className="ambient-blob blob-3" />
      <div
        className="ambient-bokeh"
        style={{ width: 22, height: 22, top: "16%", insetInlineStart: "32%" }}
      />
      <div
        className="ambient-bokeh"
        style={{
          width: 16,
          height: 16,
          top: "68%",
          insetInlineStart: "24%",
          animationDelay: "-3.2s",
        }}
      />
      <div
        className="ambient-bokeh"
        style={{
          width: 26,
          height: 26,
          top: "28%",
          insetInlineEnd: "14%",
          animationDelay: "-5.7s",
        }}
      />
      <div
        className="ambient-bokeh"
        style={{
          width: 18,
          height: 18,
          bottom: "18%",
          insetInlineEnd: "28%",
          animationDelay: "-1.8s",
        }}
      />

      <div className="app-shell">
        {/* Sidebar */}
        <aside className="sidebar" aria-label="Main Navigation">
          <div className="brand-box">
            <div className="brand-orb">
              <Sparkles size={17} />
            </div>
            <div className="brand-title">{t.appName}</div>
          </div>

          <div className="nav-scroll">
            <div className="nav-group">
              <div className="nav-group-label">{t.sidebarGroupMain}</div>
              {mainGroupNav.map((item) => (
                <button
                  key={item.id}
                  type="button"
                  className={`nav-btn ${activeNav === item.id ? "active" : ""}`}
                  onClick={() => setActiveNav(item.id)}
                >
                  <span className="nav-btn-inner">
                    {item.icon}
                    <span>{t.nav[item.id]}</span>
                  </span>
                  {item.badge && (
                    <span className="nav-pill-count">{item.badge}</span>
                  )}
                </button>
              ))}
            </div>

            <div className="nav-group">
              <div className="nav-group-label">{t.sidebarGroupFiles}</div>
              {secondaryGroupNav.map((item) => (
                <button
                  key={item.id}
                  type="button"
                  className={`nav-btn ${activeNav === item.id ? "active" : ""}`}
                  onClick={() => setActiveNav(item.id)}
                >
                  <span className="nav-btn-inner">
                    {item.icon}
                    <span>{t.nav[item.id]}</span>
                  </span>
                  {item.badge && (
                    <span className="nav-pill-count">{item.badge}</span>
                  )}
                </button>
              ))}
            </div>

            {/* Independent Settings & Updates Link */}
            <div className="nav-group">
              <div className="nav-group-label">
                {lang === "ar" ? "الإعدادات والتحديثات المستقلة" : "SYSTEM & UPDATES"}
              </div>
              <button
                type="button"
                className={`nav-btn ${activeNav === "settings" ? "active" : ""}`}
                onClick={() => setActiveNav("settings")}
              >
                <span className="nav-btn-inner">
                  <Settings size={17} />
                  <span>{t.nav.settings}</span>
                </span>
                <span className="nav-pill-count">
                  v{cloudConfig?.current_version || status?.app_version || "0.4.0"}
                </span>
              </button>
            </div>
          </div>

          {/* Live Disk Mini-Dock at Bottom of Sidebar */}
          {status && status.volumes.length > 0 && (
            <div className="sidebar-disk-dock">
              {status.volumes.map((vol) => (
                <div
                  key={vol.mount_point}
                  className="mini-disk-item"
                  onClick={() => void triggerScanOnPath(vol.mount_point, true)}
                  title={`${vol.mount_point} — ${formatBytes(vol.free_bytes)} ${t.overview.freeSpace}`}
                >
                  <div className="mini-disk-head">
                    <span className="mini-disk-name">
                      Disk {vol.mount_point.replace("\\", "")}
                    </span>
                    <span className="mini-disk-cap">
                      {formatBytes(vol.used_bytes)} / {formatBytes(vol.total_bytes)}
                    </span>
                  </div>
                  <div className="mini-disk-bar">
                    <div
                      className="mini-disk-fill"
                      style={{ width: `${Math.min(vol.usage_percent, 100)}%` }}
                    />
                  </div>
                </div>
              ))}
            </div>
          )}

          <div className="sidebar-footer-bar">
            <button
              type="button"
              className="icon-pill-btn"
              onClick={() => void handleToggleLanguage()}
            >
              <span>{t.header.languageToggle}</span>
            </button>

            <button
              type="button"
              className="icon-pill-btn"
              onClick={() => void handleToggleTheme()}
              title={theme === "dark" ? t.header.themeLight : t.header.themeDark}
            >
              {theme === "dark" ? <Sun size={14} /> : <Moon size={14} />}
            </button>
          </div>
        </aside>

        {/* Main Workspace */}
        <main className="workspace-area">
          <div className="content-scroll">
            {error && (
              <div
                className="glass-panel"
                style={{
                  borderColor: "var(--status-danger)",
                  background: "rgba(239, 68, 68, 0.12)",
                }}
                role="alert"
              >
                <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
                  <AlertTriangle size={18} color="var(--status-danger)" />
                  <strong>
                    [{error.error_code}]{" "}
                    {lang === "ar" ? error.message_ar : error.message_en}
                  </strong>
                </div>
                {error.technical_hint && (
                  <span className="mono-cell">{error.technical_hint}</span>
                )}
              </div>
            )}

            {/* ============================================================
                VIEW 1: SMART OVERVIEW BENTO DASHBOARD
               ============================================================ */}
            {activeNav === "overview" && status && (
              <>
                <div className="overview-top-grid">
                  {/* Left Hero Bento Card */}
                  <div className="hero-bento-card">
                    <div>
                      <div className="status-pulse-line">
                        <span className="pulse-dot" />
                        <span>
                          {activeJobId
                            ? t.overview.scanningBadge
                            : t.overview.readyBadge}
                        </span>
                      </div>

                      <div className="hero-metric-wrap">
                        <div className="hero-kicker">
                          {scanResult
                            ? `${t.overview.heroHeadlineScanned} (${scanResult.target_path})`
                            : t.overview.heroHeadlineNoScan}
                        </div>
                        <div className="hero-giant-number">
                          <span className="hero-giant-val">{heroParts.value}</span>
                          <span className="hero-giant-unit">{heroParts.unit}</span>
                        </div>
                        <p className="hero-desc">
                          {scanResult
                            ? t.overview.heroDescScanned
                            : t.overview.heroDescNoScan}
                        </p>
                      </div>

                      {/* Quick Scan Preset Chips */}
                      {presets.length > 0 && (
                        <div className="preset-chips-row">
                          {presets.map((p) => (
                            <button
                              key={p.id}
                              type="button"
                              className="preset-chip"
                              onClick={() => void triggerScanOnPath(p.path, false)}
                            >
                              {lang === "ar" ? p.label_ar : p.label_en}
                            </button>
                          ))}
                        </div>
                      )}
                    </div>

                    <div className="hero-cta-row">
                      <button
                        type="button"
                        className="orb-cta-btn"
                        onClick={() =>
                          void triggerScanOnPath(
                            targetPathInput ||
                              presets[0]?.path ||
                              status.volumes[0]?.mount_point ||
                              "C:\\",
                            true
                          )
                        }
                      >
                        <div className="orb-circle">
                          <Sparkles size={18} />
                        </div>
                        <div className="orb-cta-text">
                          <span className="orb-cta-title">
                            {t.overview.startScanCta}
                          </span>
                          <span className="orb-cta-sub">
                            {scanResult
                              ? `${scanResult.files_count.toLocaleString()} files`
                              : targetPathInput || "Select path"}
                          </span>
                        </div>
                      </button>

                      <button
                        type="button"
                        className="hero-link-btn"
                        onClick={() => setActiveNav("storage")}
                      >
                        {t.overview.exploreMapCta}
                      </button>
                    </div>
                  </div>

                  {/* Right Space Map Preview Card (Floating Spheres) */}
                  <div className="spacemap-preview-card">
                    <div className="spacemap-head">
                      <h3>{t.overview.spaceMapTitle}</h3>
                      <button
                        type="button"
                        className="hero-link-btn"
                        onClick={() => setActiveNav("storage")}
                      >
                        {t.overview.spaceMapOpen}
                      </button>
                    </div>

                    <div className="bubbles-stage">
                      {previewSpheres.map((sphere, idx) => (
                        <div
                          key={sphere.key}
                          className={`space-sphere tint-${idx % 4}`}
                          style={{
                            width: `${sphere.diameter}px`,
                            height: `${sphere.diameter}px`,
                            animationDelay: sphere.delay,
                          }}
                          onClick={() => {
                            if (sphere.name !== "[Files in Root]") {
                              void triggerScanOnPath(sphere.path, true);
                            } else {
                              setActiveNav("storage");
                            }
                          }}
                          title={`${sphere.path} (${sphere.sizeText})`}
                        >
                          <span className="sphere-name">{sphere.name}</span>
                          <span className="sphere-size">{sphere.sizeText}</span>
                        </div>
                      ))}
                    </div>

                    <div className="spacemap-foot">
                      <span className="pulse-dot" />
                      <span>
                        {activeJobId && scanProgress
                          ? `${scanProgress.files_processed.toLocaleString()} ${
                              t.analyzer.filesSeen
                            }...`
                          : t.overview.spaceMapHint}
                      </span>
                    </div>
                  </div>
                </div>

                {/* Bottom Row: 4 Vibrant Bento Cards */}
                <div className="bento-cards-row">
                  {/* Card 1: Space Map & Disk Engine */}
                  <div
                    className="vibrant-card emerald"
                    onClick={() => setActiveNav("storage")}
                  >
                    <span
                      className="sparkle-dot"
                      style={{ top: "24%", insetInlineEnd: "18%" }}
                    />
                    <span
                      className="sparkle-dot"
                      style={{
                        bottom: "28%",
                        insetInlineEnd: "32%",
                        animationDelay: "-1.5s",
                      }}
                    />
                    <div>
                      <div className="vibrant-card-head">
                        <div className="vibrant-icon-box">
                          <HardDrive size={17} />
                        </div>
                        <span>{t.overview.bentoStorageTitle}</span>
                      </div>
                      <div className="vibrant-metric">
                        {scanResult
                          ? scanResult.folders_count.toLocaleString()
                          : status.volumes.length}
                        <span>
                          {scanResult
                            ? t.analyzer.foldersSeen
                            : lang === "ar"
                            ? "أقراص"
                            : "drives"}
                        </span>
                      </div>
                      <p className="vibrant-sub">{t.overview.bentoStorageSub}</p>
                    </div>
                    <button type="button" className="vibrant-pill-btn">
                      {t.overview.bentoStorageBtn}
                    </button>
                  </div>

                  {/* Card 2: Large Files */}
                  <div
                    className="vibrant-card ocean"
                    onClick={() => setActiveNav("large_files")}
                  >
                    <span
                      className="sparkle-dot"
                      style={{ top: "30%", insetInlineEnd: "22%" }}
                    />
                    <div>
                      <div className="vibrant-card-head">
                        <div className="vibrant-icon-box">
                          <FolderSearch size={17} />
                        </div>
                        <span>{t.overview.bentoLargeTitle}</span>
                      </div>
                      <div className="vibrant-metric">
                        {scanResult ? scanResult.top_files.length : 0}
                        <span>{lang === "ar" ? "ملف ضخم" : "top files"}</span>
                      </div>
                      <p className="vibrant-sub">{t.overview.bentoLargeSub}</p>
                    </div>
                    <button type="button" className="vibrant-pill-btn">
                      {t.overview.bentoLargeBtn}
                    </button>
                  </div>

                  {/* Card 3: Sentinel Security */}
                  <div
                    className="vibrant-card indigo"
                    onClick={() => setActiveNav("settings")}
                  >
                    <span
                      className="sparkle-dot"
                      style={{ top: "20%", insetInlineEnd: "25%" }}
                    />
                    <span
                      className="sparkle-dot"
                      style={{
                        bottom: "22%",
                        insetInlineStart: "45%",
                        animationDelay: "-2s",
                      }}
                    />
                    <div>
                      <div className="vibrant-card-head">
                        <div className="vibrant-icon-box">
                          <ShieldCheck size={17} />
                        </div>
                        <span>{t.overview.bentoSentinelTitle}</span>
                      </div>
                      <div className="vibrant-metric">
                        ABI v1
                        <span>READY</span>
                      </div>
                      <p className="vibrant-sub">{t.overview.bentoSentinelSub}</p>
                    </div>
                    <button type="button" className="vibrant-pill-btn">
                      {t.overview.bentoSentinelBtn}
                    </button>
                  </div>

                  {/* Card 4: Safe Cleanup Policy */}
                  <div
                    className="vibrant-card coral"
                    onClick={() => setActiveNav("cleanup")}
                  >
                    <span
                      className="sparkle-dot"
                      style={{ top: "26%", insetInlineEnd: "16%" }}
                    />
                    <div>
                      <div className="vibrant-card-head">
                        <div className="vibrant-icon-box">
                          <Sparkles size={17} />
                        </div>
                        <span>{t.overview.bentoCleanupTitle}</span>
                      </div>
                      <div className="vibrant-metric">
                        100%
                        <span>{lang === "ar" ? "آمن" : "safe policy"}</span>
                      </div>
                      <p className="vibrant-sub">{t.overview.bentoCleanupSub}</p>
                    </div>
                    <button type="button" className="vibrant-pill-btn">
                      {t.overview.bentoCleanupBtn}
                    </button>
                  </div>
                </div>
              </>
            )}

            {/* ============================================================
                VIEW 2: STORAGE ANALYZER & INTERACTIVE SPACE MAP (PHASE 1)
               ============================================================ */}
            {activeNav === "storage" && (
              <>
                <div className="glass-panel">
                  <div className="panel-header-row">
                    <div>
                      <h2 style={{ fontSize: 20, fontWeight: 800 }}>
                        {t.analyzer.title}
                      </h2>
                      <p style={{ fontSize: 13, color: "var(--text-secondary)" }}>
                        {t.analyzer.subtitle}
                      </p>
                    </div>

                    {scanResult && (
                      <button
                        type="button"
                        className="icon-pill-btn"
                        onClick={handleNavigateParent}
                      >
                        <ArrowUp size={14} />
                        <span>{t.analyzer.upOneLevel}</span>
                      </button>
                    )}
                  </div>

                  {/* Path Input & Quick Presets */}
                  <div className="scan-bar-row">
                    <input
                      type="text"
                      className="glass-input mono-cell"
                      value={targetPathInput}
                      onChange={(e) => setTargetPathInput(e.target.value)}
                      placeholder={t.analyzer.customPathPlaceholder}
                    />
                    {activeJobId ? (
                      <button
                        type="button"
                        className="danger-pill-btn"
                        onClick={() => void handleCancelScan()}
                      >
                        <Square size={14} />
                        <span>{t.analyzer.cancelBtn}</span>
                      </button>
                    ) : (
                      <button
                        type="button"
                        className="primary-pill-btn"
                        onClick={() =>
                          void triggerScanOnPath(targetPathInput, false)
                        }
                      >
                        <Play size={14} />
                        <span>{t.analyzer.scanBtn}</span>
                      </button>
                    )}
                  </div>

                  {presets.length > 0 && (
                    <div className="preset-chips-row" style={{ marginTop: 2 }}>
                      {presets.map((p) => (
                        <button
                          key={p.id}
                          type="button"
                          className="preset-chip"
                          onClick={() => void triggerScanOnPath(p.path, false)}
                        >
                          {lang === "ar" ? p.label_ar : p.label_en}
                        </button>
                      ))}
                    </div>
                  )}
                </div>

                {/* Live Active Scan Telemetry Banner */}
                {activeJobId && scanProgress && (
                  <div className="glass-panel">
                    <div className="panel-header-row">
                      <div className="status-pulse-line">
                        <span className="pulse-dot" />
                        <span>{t.analyzer.scanningTitle}</span>
                      </div>
                      <span className="mono-cell">
                        {scanProgress.files_processed.toLocaleString()}{" "}
                        {t.analyzer.filesSeen} •{" "}
                        {scanProgress.folders_processed.toLocaleString()}{" "}
                        {t.analyzer.foldersSeen} •{" "}
                        {formatBytes(scanProgress.bytes_processed)}
                      </span>
                    </div>
                    <div
                      className="mono-cell"
                      style={{
                        color: "var(--text-secondary)",
                        overflow: "hidden",
                        textOverflow: "ellipsis",
                        whiteSpace: "nowrap",
                      }}
                    >
                      {scanProgress.current_path}
                    </div>
                  </div>
                )}

                {!scanResult && !activeJobId && (
                  <div
                    className="glass-panel"
                    style={{ textAlign: "center", paddingBlock: 48 }}
                  >
                    <HardDrive
                      size={40}
                      color="var(--accent-primary)"
                      style={{ marginInline: "auto" }}
                    />
                    <h3 style={{ marginTop: 8 }}>{t.analyzer.emptyStateTitle}</h3>
                    <p style={{ color: "var(--text-secondary)" }}>
                      {t.analyzer.emptyStateDesc}
                    </p>
                  </div>
                )}

                {scanResult && (
                  <>
                    {/* Partial Scan Warning Banner if errors_count > 0 */}
                    {scanResult.is_partial && (
                      <div
                        className="glass-panel"
                        style={{
                          borderColor: "var(--status-warn)",
                          background: "rgba(245, 158, 11, 0.1)",
                        }}
                      >
                        <div
                          style={{
                            display: "flex",
                            alignItems: "center",
                            gap: 10,
                          }}
                        >
                          <AlertTriangle size={18} color="var(--status-warn)" />
                          <strong>
                            {t.analyzer.partialScanTitle} (
                            {scanResult.errors_count} items inaccessible)
                          </strong>
                        </div>
                        <p style={{ fontSize: 13, color: "var(--text-secondary)" }}>
                          {t.analyzer.partialScanDesc}
                        </p>
                      </div>
                    )}

                    {/* Summary Metrics Strip */}
                    <div
                      style={{
                        display: "grid",
                        gridTemplateColumns: "repeat(auto-fit, minmax(190px, 1fr))",
                        gap: 14,
                      }}
                    >
                      <div className="glass-panel" style={{ padding: 16 }}>
                        <span
                          style={{ fontSize: 12, color: "var(--text-secondary)" }}
                        >
                          {t.analyzer.logicalTotal}
                        </span>
                        <strong
                          className="mono-cell"
                          style={{ fontSize: 22, color: "var(--accent-primary)" }}
                        >
                          {formatBytes(scanResult.total_logical_bytes)}
                        </strong>
                      </div>
                      <div className="glass-panel" style={{ padding: 16 }}>
                        <span
                          style={{ fontSize: 12, color: "var(--text-secondary)" }}
                        >
                          {t.analyzer.allocatedTotal}
                        </span>
                        <strong className="mono-cell" style={{ fontSize: 22 }}>
                          {formatBytes(scanResult.total_allocated_bytes)}
                        </strong>
                      </div>
                      <div className="glass-panel" style={{ padding: 16 }}>
                        <span
                          style={{ fontSize: 12, color: "var(--text-secondary)" }}
                        >
                          {t.analyzer.filesSeen} / {t.analyzer.foldersSeen}
                        </span>
                        <strong className="mono-cell" style={{ fontSize: 20 }}>
                          {scanResult.files_count.toLocaleString()} /{" "}
                          {scanResult.folders_count.toLocaleString()}
                        </strong>
                      </div>
                      <div className="glass-panel" style={{ padding: 16 }}>
                        <span
                          style={{ fontSize: 12, color: "var(--text-secondary)" }}
                        >
                          {t.analyzer.reparseSkipped}
                        </span>
                        <strong className="mono-cell" style={{ fontSize: 20 }}>
                          {scanResult.reparse_points_skipped} (
                          {scanResult.duration_ms} ms)
                        </strong>
                      </div>
                    </div>

                    {/* Interactive Space Map (Toggle Bubbles vs Treemap Grid) */}
                    <div className="glass-panel">
                      <div className="panel-header-row">
                        <div className="panel-title">
                          <Layers size={18} color="var(--accent-primary)" />
                          <span>
                            {t.analyzer.spaceMapSection} —{" "}
                            <span className="mono-cell">
                              {scanResult.target_path}
                            </span>
                          </span>
                        </div>

                        <div style={{ display: "flex", gap: 8 }}>
                          <button
                            type="button"
                            className="icon-pill-btn"
                            style={{
                              borderColor:
                                mapMode === "bubbles"
                                  ? "var(--accent-primary)"
                                  : undefined,
                            }}
                            onClick={() => setMapMode("bubbles")}
                          >
                            <span>{t.analyzer.viewBubbles}</span>
                          </button>
                          <button
                            type="button"
                            className="icon-pill-btn"
                            style={{
                              borderColor:
                                mapMode === "treemap"
                                  ? "var(--accent-primary)"
                                  : undefined,
                            }}
                            onClick={() => setMapMode("treemap")}
                          >
                            <span>{t.analyzer.viewTreemap}</span>
                          </button>
                        </div>
                      </div>

                      {mapMode === "bubbles" ? (
                        <div
                          className="bubbles-stage"
                          style={{ minHeight: 240, paddingBlock: 18 }}
                        >
                          {scanResult.immediate_children.slice(0, 12).map((child, idx) => {
                            const maxSz = Math.max(
                              ...scanResult.immediate_children.map(
                                (c) => c.logical_size
                              ),
                              1
                            );
                            const ratio = Math.sqrt(child.logical_size / maxSz);
                            const diameter = Math.round(74 + ratio * 66);
                            return (
                              <div
                                key={child.path}
                                className={`space-sphere tint-${idx % 4}`}
                                style={{
                                  width: `${diameter}px`,
                                  height: `${diameter}px`,
                                  animationDelay: `${idx * -0.6}s`,
                                }}
                                onClick={() => {
                                  if (child.folder_name !== "[Files in Root]") {
                                    void triggerScanOnPath(child.path, false);
                                  }
                                }}
                                title={`${child.path}\nLogical: ${formatBytes(
                                  child.logical_size
                                )}\nAllocated: ${formatBytes(child.allocated_size)}`}
                              >
                                <span className="sphere-name">
                                  {child.folder_name}
                                </span>
                                <span className="sphere-size">
                                  {formatBytes(child.logical_size)}
                                </span>
                                <span
                                  style={{
                                    fontSize: 10,
                                    color: "var(--text-secondary)",
                                  }}
                                >
                                  {child.percentage_of_root}%
                                </span>
                              </div>
                            );
                          })}
                        </div>
                      ) : (
                        <div className="treemap-grid">
                          {scanResult.immediate_children.slice(0, 18).map((child) => (
                            <div
                              key={child.path}
                              className="treemap-tile"
                              onClick={() => {
                                if (child.folder_name !== "[Files in Root]") {
                                  void triggerScanOnPath(child.path, false);
                                }
                              }}
                            >
                              <div
                                style={{
                                  display: "flex",
                                  justifyContent: "space-between",
                                  gap: 8,
                                }}
                              >
                                <strong
                                  style={{
                                    overflow: "hidden",
                                    textOverflow: "ellipsis",
                                    whiteSpace: "nowrap",
                                  }}
                                >
                                  {child.folder_name}
                                </strong>
                                {child.is_partial && (
                                  <span className="badge-warn">Partial</span>
                                )}
                              </div>
                              <div>
                                <div
                                  className="mono-cell"
                                  style={{
                                    fontSize: 15,
                                    fontWeight: 800,
                                    color: "var(--accent-primary)",
                                  }}
                                >
                                  {formatBytes(child.logical_size)}
                                </div>
                                <div
                                  style={{
                                    fontSize: 11,
                                    color: "var(--text-secondary)",
                                  }}
                                >
                                  {child.percentage_of_root}% •{" "}
                                  {child.files_count.toLocaleString()} files
                                </div>
                              </div>
                              <div
                                className="treemap-tile-bar"
                                style={{
                                  width: `${Math.min(
                                    child.percentage_of_root,
                                    100
                                  )}%`,
                                }}
                              />
                            </div>
                          ))}
                        </div>
                      )}
                    </div>

                    {/* Category & Extension Breakdown */}
                    <div
                      style={{
                        display: "grid",
                        gridTemplateColumns: "1fr 1fr",
                        gap: 16,
                      }}
                    >
                      <div className="glass-panel">
                        <div className="panel-title">
                          <span>{t.analyzer.categoriesSection}</span>
                        </div>
                        <div
                          style={{
                            display: "flex",
                            flexDirection: "column",
                            gap: 10,
                          }}
                        >
                          {scanResult.category_breakdown.map((cat) => (
                            <div
                              key={cat.category}
                              style={{
                                display: "flex",
                                flexDirection: "column",
                                gap: 4,
                              }}
                            >
                              <div
                                style={{
                                  display: "flex",
                                  justifyContent: "space-between",
                                  fontSize: 12.5,
                                }}
                              >
                                <strong>{cat.category}</strong>
                                <span className="mono-cell">
                                  {formatBytes(cat.logical_size)} ({cat.percentage}
                                  %)
                                </span>
                              </div>
                              <div className="mini-disk-bar">
                                <div
                                  className="mini-disk-fill"
                                  style={{
                                    width: `${Math.min(cat.percentage, 100)}%`,
                                  }}
                                />
                              </div>
                            </div>
                          ))}
                        </div>
                      </div>

                      <div className="glass-panel">
                        <div className="panel-title">
                          <span>{t.analyzer.extensionsSection}</span>
                        </div>
                        <div
                          style={{
                            display: "flex",
                            flexDirection: "column",
                            gap: 8,
                          }}
                        >
                          {scanResult.extension_breakdown.slice(0, 8).map((ext) => (
                            <div
                              key={ext.extension}
                              style={{
                                display: "flex",
                                justifyContent: "space-between",
                                alignItems: "center",
                                padding: "6px 10px",
                                borderRadius: 10,
                                background: "var(--glass-card-inner)",
                              }}
                            >
                              <div>
                                <strong className="mono-cell">
                                  .{ext.extension}
                                </strong>{" "}
                                <span
                                  style={{
                                    fontSize: 11.5,
                                    color: "var(--text-secondary)",
                                  }}
                                >
                                  ({ext.files_count.toLocaleString()} files)
                                </span>
                              </div>
                              <span
                                className="mono-cell"
                                style={{ fontWeight: 700 }}
                              >
                                {formatBytes(ext.logical_size)} ({ext.percentage}%)
                              </span>
                            </div>
                          ))}
                        </div>
                      </div>
                    </div>

                    {/* Discovered Subfolders Table with Search, Sort, Explorer & Copy */}
                    <div className="glass-panel">
                      <div className="panel-header-row">
                        <div className="panel-title">
                          <FolderOpen size={18} color="var(--accent-primary)" />
                          <span>{t.analyzer.foldersSection}</span>
                        </div>

                        <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
                          <input
                            type="text"
                            className="glass-input"
                            style={{ minWidth: 200, padding: "6px 12px" }}
                            value={folderSearch}
                            onChange={(e) => setFolderSearch(e.target.value)}
                            placeholder={t.analyzer.searchPlaceholder}
                          />
                          <select
                            className="icon-pill-btn"
                            value={folderSort}
                            onChange={(e) =>
                              setFolderSort(
                                e.target.value as
                                  | "logical"
                                  | "allocated"
                                  | "files"
                                  | "name"
                              )
                            }
                          >
                            <option value="logical">
                              {t.analyzer.sortByLogical}
                            </option>
                            <option value="allocated">
                              {t.analyzer.sortByAllocated}
                            </option>
                            <option value="files">{t.analyzer.sortByFiles}</option>
                            <option value="name">{t.analyzer.sortByName}</option>
                          </select>
                        </div>
                      </div>

                      <table className="data-table">
                        <thead>
                          <tr>
                            <th>Folder</th>
                            <th>{t.analyzer.logicalTotal}</th>
                            <th>{t.analyzer.allocatedTotal}</th>
                            <th>Files</th>
                            <th>Actions</th>
                          </tr>
                        </thead>
                        <tbody>
                          {displayedFolders.slice(0, 50).map((f) => (
                            <tr key={f.path}>
                              <td>
                                <div style={{ fontWeight: 700 }}>
                                  {f.folder_name}{" "}
                                  {f.is_partial && (
                                    <span className="badge-warn">Partial</span>
                                  )}
                                </div>
                                <div
                                  className="mono-cell"
                                  style={{
                                    color: "var(--text-secondary)",
                                    fontSize: 11,
                                  }}
                                >
                                  {f.path}
                                </div>
                              </td>
                              <td className="mono-cell" style={{ fontWeight: 700 }}>
                                {formatBytes(f.logical_size)}
                              </td>
                              <td className="mono-cell">
                                {formatBytes(f.allocated_size)}
                              </td>
                              <td className="mono-cell">
                                {f.files_count.toLocaleString()}
                              </td>
                              <td>
                                <div style={{ display: "flex", gap: 6 }}>
                                  <button
                                    type="button"
                                    className="icon-pill-btn"
                                    onClick={() =>
                                      void triggerScanOnPath(f.path, false)
                                    }
                                    title={t.analyzer.drillDown}
                                  >
                                    <Play size={12} />
                                  </button>
                                  <button
                                    type="button"
                                    className="icon-pill-btn"
                                    onClick={() =>
                                      void openPathInExplorer(f.path)
                                    }
                                    title={t.analyzer.openExplorer}
                                  >
                                    <ExternalLink size={12} />
                                  </button>
                                  <button
                                    type="button"
                                    className="icon-pill-btn"
                                    onClick={() => void handleCopyPath(f.path)}
                                    title={t.analyzer.copyPath}
                                  >
                                    {copiedPath === f.path ? (
                                      <Check size={12} />
                                    ) : (
                                      <Copy size={12} />
                                    )}
                                  </button>
                                </div>
                              </td>
                            </tr>
                          ))}
                        </tbody>
                      </table>
                    </div>
                  </>
                )}
              </>
            )}

            {/* ============================================================
                VIEW 3: DEEP LARGE FILES INTELLIGENCE & SAFE REMOVAL
               ============================================================ */}
            {activeNav === "large_files" && (
              <>
                <div className="glass-panel">
                  <div className="panel-header-row">
                    <div>
                      <h2 style={{ fontSize: 20, fontWeight: 800 }}>
                        {t.largeFiles.title}
                      </h2>
                      <p style={{ fontSize: 13, color: "var(--text-secondary)" }}>
                        {t.largeFiles.subtitle}
                      </p>
                    </div>

                    {scanResult && (
                      <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
                        <input
                          type="text"
                          className="glass-input"
                          style={{ minWidth: 220, padding: "6px 12px" }}
                          value={largeFileSearch}
                          onChange={(e) => setLargeFileSearch(e.target.value)}
                          placeholder={t.largeFiles.searchPlaceholder}
                        />
                        <select
                          className="icon-pill-btn"
                          value={largeFileMinSize}
                          onChange={(e) =>
                            setLargeFileMinSize(Number(e.target.value))
                          }
                        >
                          <option value={0}>{t.largeFiles.allSizes}</option>
                          <option value={10485760}>&ge; 10 MB</option>
                          <option value={52428800}>&ge; 50 MB</option>
                          <option value={104857600}>&ge; 100 MB</option>
                          <option value={524288000}>&ge; 500 MB</option>
                          <option value={1073741824}>&ge; 1 GB</option>
                        </select>
                        <select
                          className="icon-pill-btn"
                          value={selectedCategory}
                          onChange={(e) => setSelectedCategory(e.target.value)}
                        >
                          <option value="ALL">
                            {t.largeFiles.allCategories}
                          </option>
                          {scanResult.category_breakdown.map((c) => (
                            <option key={c.category} value={c.category}>
                              {c.category}
                            </option>
                          ))}
                        </select>
                      </div>
                    )}
                  </div>

                  {largeDeleteReport && (
                    <div
                      className="glass-panel"
                      style={{
                        borderColor: "var(--status-ok)",
                        background: "rgba(16, 185, 129, 0.12)",
                      }}
                    >
                      <div
                        style={{
                          display: "flex",
                          alignItems: "center",
                          gap: 10,
                        }}
                      >
                        <ShieldCheck size={18} color="var(--status-ok)" />
                        <strong>
                          {t.largeFiles.deleteSuccessBanner}{" "}
                          {largeDeleteReport.succeeded_count} removed •{" "}
                          {formatBytes(largeDeleteReport.bytes_reclaimed)}{" "}
                          reclaimed
                          {largeDeleteReport.skipped_or_failed_count > 0
                            ? ` (${largeDeleteReport.skipped_or_failed_count} skipped/locked)`
                            : ""}
                        </strong>
                      </div>
                    </div>
                  )}

                  {!scanResult ? (
                    <p
                      style={{
                        paddingBlock: 28,
                        color: "var(--text-secondary)",
                      }}
                    >
                      {t.largeFiles.noScanYet}
                    </p>
                  ) : (
                    <div className="table-scroll-container">
                      <table className="data-table">
                        <thead>
                          <tr>
                            <th style={{ width: 52 }}>{t.largeFiles.colSelect}</th>
                            <th>{t.largeFiles.colName}</th>
                            <th>{t.largeFiles.colCategory}</th>
                            <th>{t.largeFiles.colLogicalSize}</th>
                            <th>{t.largeFiles.colAllocatedSize}</th>
                            <th>{t.largeFiles.colActions}</th>
                          </tr>
                        </thead>
                      <tbody>
                        {displayedLargeFiles.map((file) => {
                          const isSelected = selectedLargeFilePaths.includes(
                            file.path
                          );
                          return (
                            <tr
                              key={file.path}
                              style={
                                isSelected
                                  ? {
                                      background: "rgba(239, 68, 68, 0.08)",
                                    }
                                  : undefined
                              }
                            >
                              <td>
                                <input
                                  type="checkbox"
                                  checked={isSelected}
                                  onChange={() =>
                                    handleToggleLargeFileSelect(file.path)
                                  }
                                  style={{
                                    width: 16,
                                    height: 16,
                                    cursor: "pointer",
                                  }}
                                />
                              </td>
                              <td>
                                <div style={{ fontWeight: 700 }}>
                                  {file.file_name}
                                </div>
                                <div
                                  className="mono-cell"
                                  style={{
                                    color: "var(--text-secondary)",
                                    fontSize: 11,
                                  }}
                                >
                                  {file.path}
                                </div>
                              </td>
                              <td>
                                <span className="badge-ok">{file.category}</span>{" "}
                                <span className="mono-cell">
                                  .{file.extension}
                                </span>
                              </td>
                              <td
                                className="mono-cell"
                                style={{
                                  fontWeight: 800,
                                  color: "var(--accent-primary)",
                                }}
                              >
                                {formatBytes(file.logical_size)}
                              </td>
                              <td className="mono-cell">
                                {formatBytes(file.allocated_size)}
                              </td>
                              <td>
                                <div style={{ display: "flex", gap: 6 }}>
                                  <button
                                    type="button"
                                    className="icon-pill-btn"
                                    onClick={() =>
                                      void openPathInExplorer(file.path)
                                    }
                                    title={t.analyzer.openExplorer}
                                  >
                                    <ExternalLink size={12} />
                                  </button>
                                  <button
                                    type="button"
                                    className="icon-pill-btn"
                                    onClick={() =>
                                      void handleCopyPath(file.path)
                                    }
                                    title={t.analyzer.copyPath}
                                  >
                                    {copiedPath === file.path ? (
                                      <Check size={12} />
                                    ) : (
                                      <Copy size={12} />
                                    )}
                                  </button>
                                </div>
                              </td>
                            </tr>
                          );
                        })}
                      </tbody>
                    </table>
                    </div>
                  )}
                </div>

                {largeFileSelectionSummary.count > 0 && (
                  <div className="floating-action-dock">
                    <div>
                      <div
                        style={{
                          fontSize: 15,
                          fontWeight: 800,
                          color: "var(--text-primary)",
                        }}
                      >
                        {largeFileSelectionSummary.count}{" "}
                        {t.largeFiles.selectedCount} •{" "}
                        <span
                          className="mono-cell"
                          style={{ color: "var(--accent-primary)" }}
                        >
                          {formatBytes(largeFileSelectionSummary.bytes)}
                        </span>
                      </div>
                      {confirmingLargeDelete && (
                        <div
                          style={{
                            fontSize: 12,
                            color: "var(--status-warn)",
                            marginTop: 3,
                          }}
                        >
                          {t.largeFiles.confirmWarning}
                        </div>
                      )}
                    </div>

                    <div style={{ display: "flex", gap: 10 }}>
                      {!confirmingLargeDelete ? (
                        <button
                          type="button"
                          className="primary-pill-btn"
                          onClick={() => setConfirmingLargeDelete(true)}
                        >
                          <Trash2 size={14} />
                          <span>{t.largeFiles.reviewDeleteBtn}</span>
                        </button>
                      ) : (
                        <>
                          <button
                            type="button"
                            className="icon-pill-btn"
                            onClick={() => setConfirmingLargeDelete(false)}
                            disabled={deletingLargeFiles}
                          >
                            <span>{t.duplicates.cancelConfirmBtn}</span>
                          </button>
                          <button
                            type="button"
                            className="danger-pill-btn"
                            onClick={() => void handleExecuteLargeFileDelete()}
                            disabled={deletingLargeFiles}
                          >
                            <Trash2 size={14} />
                            <span>{t.largeFiles.confirmDeleteBtn}</span>
                          </button>
                        </>
                      )}
                    </div>
                  </div>
                )}
              </>
            )}

            {/* ============================================================
                VIEW 4: DUPLICATE FINDER (PHASE 2 — SHA-256 CONTENT ENGINE)
               ============================================================ */}
            {activeNav === "duplicates" && (
              <>
                <div className="glass-panel">
                  <div className="panel-header-row">
                    <div>
                      <h2 style={{ fontSize: 20, fontWeight: 800 }}>
                        {t.duplicates.title}
                      </h2>
                      <p style={{ fontSize: 13, color: "var(--text-secondary)" }}>
                        {t.duplicates.subtitle}
                      </p>
                    </div>
                  </div>

                  {/* Path Input, Min Size Selector & Scan Trigger */}
                  <div className="scan-bar-row">
                    <input
                      type="text"
                      className="glass-input mono-cell"
                      value={targetPathInput}
                      onChange={(e) => setTargetPathInput(e.target.value)}
                      placeholder={t.analyzer.customPathPlaceholder}
                    />
                    <div
                      style={{
                        display: "inline-flex",
                        alignItems: "center",
                        gap: 6,
                      }}
                    >
                      <span
                        style={{
                          fontSize: 12,
                          fontWeight: 700,
                          color: "var(--text-secondary)",
                        }}
                      >
                        {t.duplicates.minSizeLabel}
                      </span>
                      <select
                        className="icon-pill-btn"
                        value={dupMinSizeBytes}
                        onChange={(e) =>
                          setDupMinSizeBytes(Number(e.target.value))
                        }
                      >
                        <option value={1024}>1 KB</option>
                        <option value={65536}>64 KB</option>
                        <option value={1048576}>1 MB</option>
                        <option value={10485760}>10 MB</option>
                      </select>
                    </div>

                    {activeDupJobId ? (
                      <button
                        type="button"
                        className="danger-pill-btn"
                        onClick={() => void handleCancelDupScan()}
                      >
                        <Square size={14} />
                        <span>{t.duplicates.cancelBtn}</span>
                      </button>
                    ) : (
                      <button
                        type="button"
                        className="primary-pill-btn"
                        onClick={() =>
                          void triggerDuplicateScan(targetPathInput)
                        }
                      >
                        <Play size={14} />
                        <span>{t.duplicates.scanBtn}</span>
                      </button>
                    )}
                  </div>

                  {presets.length > 0 && (
                    <div className="preset-chips-row" style={{ marginTop: 2 }}>
                      {presets.map((p) => (
                        <button
                          key={p.id}
                          type="button"
                          className="preset-chip"
                          onClick={() => void triggerDuplicateScan(p.path)}
                        >
                          {lang === "ar" ? p.label_ar : p.label_en}
                        </button>
                      ))}
                    </div>
                  )}
                </div>

                {/* Live Duplicate Scan Progress */}
                {activeDupJobId && dupProgress && (
                  <div className="glass-panel">
                    <div className="panel-header-row">
                      <div className="status-pulse-line">
                        <span className="pulse-dot" />
                        <span>{t.duplicates.scanningTitle}</span>
                      </div>
                      <span className="mono-cell">
                        {dupProgress.files_processed.toLocaleString()}{" "}
                        {t.analyzer.filesSeen} •{" "}
                        {formatBytes(dupProgress.bytes_processed)}
                      </span>
                    </div>
                    <div
                      className="mono-cell"
                      style={{
                        color: "var(--text-secondary)",
                        overflow: "hidden",
                        textOverflow: "ellipsis",
                        whiteSpace: "nowrap",
                      }}
                    >
                      {dupProgress.current_path}
                    </div>
                  </div>
                )}

                {/* Duplicate Removal Completion Banner */}
                {dupRemovalReport && (
                  <div
                    className="glass-panel"
                    style={{
                      borderColor: "var(--status-ok)",
                      background: "rgba(16, 185, 129, 0.12)",
                    }}
                  >
                    <div
                      style={{
                        display: "flex",
                        alignItems: "center",
                        gap: 10,
                      }}
                    >
                      <ShieldCheck size={18} color="var(--status-ok)" />
                      <strong>
                        {t.duplicates.removalSuccessBanner}{" "}
                        {dupRemovalReport.succeeded_count} removed •{" "}
                        {formatBytes(dupRemovalReport.bytes_reclaimed)} reclaimed
                        {dupRemovalReport.failed_count > 0
                          ? ` (${dupRemovalReport.failed_count} skipped/failed)`
                          : ""}
                      </strong>
                    </div>
                  </div>
                )}

                {!dupResult && !activeDupJobId && (
                  <div
                    className="glass-panel"
                    style={{ textAlign: "center", paddingBlock: 48 }}
                  >
                    <Layers
                      size={40}
                      color="var(--accent-primary)"
                      style={{ marginInline: "auto" }}
                    />
                    <h3 style={{ marginTop: 8 }}>
                      {t.duplicates.emptyStateTitle}
                    </h3>
                    <p style={{ color: "var(--text-secondary)" }}>
                      {t.duplicates.emptyStateDesc}
                    </p>
                  </div>
                )}

                {dupResult && (
                  <>
                    {/* Summary Metrics Strip */}
                    <div
                      style={{
                        display: "grid",
                        gridTemplateColumns:
                          "repeat(auto-fit, minmax(195px, 1fr))",
                        gap: 14,
                      }}
                    >
                      <div className="glass-panel" style={{ padding: 16 }}>
                        <span
                          style={{
                            fontSize: 12,
                            color: "var(--text-secondary)",
                          }}
                        >
                          {t.duplicates.groupsCount}
                        </span>
                        <strong
                          className="mono-cell"
                          style={{
                            fontSize: 22,
                            color: "var(--accent-primary)",
                          }}
                        >
                          {dupResult.total_duplicate_groups.toLocaleString()} (
                          {dupResult.total_duplicate_files.toLocaleString()}{" "}
                          files)
                        </strong>
                      </div>
                      <div className="glass-panel" style={{ padding: 16 }}>
                        <span
                          style={{
                            fontSize: 12,
                            color: "var(--text-secondary)",
                          }}
                        >
                          {t.duplicates.reclaimableSpace}
                        </span>
                        <strong
                          className="mono-cell"
                          style={{ fontSize: 22, color: "var(--status-ok)" }}
                        >
                          {formatBytes(dupResult.total_reclaimable_bytes)}
                        </strong>
                      </div>
                      <div className="glass-panel" style={{ padding: 16 }}>
                        <span
                          style={{
                            fontSize: 12,
                            color: "var(--text-secondary)",
                          }}
                        >
                          {t.duplicates.fullHashesVerified}
                        </span>
                        <strong className="mono-cell" style={{ fontSize: 20 }}>
                          {dupResult.full_hashes_computed.toLocaleString()} /{" "}
                          {dupResult.partial_hashes_computed.toLocaleString()}{" "}
                          partial ({dupResult.duration_ms} ms)
                        </strong>
                      </div>
                    </div>

                    {dupResult.groups.length === 0 ? (
                      <div
                        className="glass-panel"
                        style={{ textAlign: "center", paddingBlock: 42 }}
                      >
                        <ShieldCheck
                          size={38}
                          color="var(--status-ok)"
                          style={{ marginInline: "auto" }}
                        />
                        <h3 style={{ marginTop: 8 }}>
                          {t.duplicates.noDuplicatesFoundTitle}
                        </h3>
                        <p style={{ color: "var(--text-secondary)" }}>
                          {t.duplicates.noDuplicatesFoundDesc}
                        </p>
                      </div>
                    ) : (
                      <div className="glass-panel">
                        {/* Smart Selection Toolbar */}
                        <div className="panel-header-row">
                          <div className="panel-title">
                            <Layers size={18} color="var(--accent-primary)" />
                            <span>
                              SHA-256 {t.duplicates.groupsCount} —{" "}
                              <span className="mono-cell">
                                {dupResult.target_path}
                              </span>
                            </span>
                          </div>

                          <div
                            style={{
                              display: "flex",
                              gap: 8,
                              flexWrap: "wrap",
                            }}
                          >
                            <button
                              type="button"
                              className="icon-pill-btn"
                              onClick={() =>
                                handleSmartSelectDuplicates("keep_oldest")
                              }
                            >
                              <Sparkles size={13} />
                              <span>{t.duplicates.smartSelectOldest}</span>
                            </button>
                            <button
                              type="button"
                              className="icon-pill-btn"
                              onClick={() =>
                                handleSmartSelectDuplicates("keep_newest")
                              }
                            >
                              <Sparkles size={13} />
                              <span>{t.duplicates.smartSelectNewest}</span>
                            </button>
                            <button
                              type="button"
                              className="icon-pill-btn"
                              onClick={handleClearDuplicateSelection}
                            >
                              <span>{t.duplicates.clearSelection}</span>
                            </button>
                          </div>
                        </div>

                        {/* Duplicate Groups List */}
                        <div
                          style={{
                            display: "flex",
                            flexDirection: "column",
                            gap: 14,
                          }}
                        >
                          {dupResult.groups.map((grp) => {
                            const retainedPath =
                              retainedByGroup[grp.group_id] ||
                              grp.copies[0]?.path ||
                              "";
                            const selectedDeletes =
                              selectedDeletesByGroup[grp.group_id] || [];
                            const hasHardlinks = grp.copies.some(
                              (c) => c.is_hardlink_alias || c.hardlink_count > 1
                            );

                            return (
                              <div key={grp.group_id} className="dup-group-card">
                                <div className="dup-group-header">
                                  <div
                                    style={{
                                      display: "flex",
                                      alignItems: "center",
                                      gap: 10,
                                      flexWrap: "wrap",
                                    }}
                                  >
                                    <span className="badge-ok">
                                      {grp.copies.length}{" "}
                                      {t.duplicates.copiesBadge}
                                    </span>
                                    <strong className="mono-cell">
                                      {t.duplicates.eachFileSize}:{" "}
                                      {formatBytes(grp.file_size_bytes)}
                                    </strong>
                                    <span
                                      className="mono-cell"
                                      style={{
                                        color: "var(--accent-primary)",
                                        fontWeight: 700,
                                      }}
                                    >
                                      {t.duplicates.reclaimableSpace}:{" "}
                                      {formatBytes(grp.wasted_bytes)}
                                    </span>
                                    {hasHardlinks && (
                                      <span
                                        className="badge-warn"
                                        title={t.duplicates.hardlinkWarning}
                                      >
                                        {t.duplicates.hardlinkBadge}
                                      </span>
                                    )}
                                  </div>

                                  <span
                                    className="mono-cell"
                                    style={{
                                      fontSize: 11,
                                      color: "var(--text-secondary)",
                                    }}
                                    title={`SHA-256: ${grp.sha256}`}
                                  >
                                    SHA-256: {grp.sha256.slice(0, 16)}…
                                    {grp.sha256.slice(-8)}
                                  </span>
                                </div>

                                {hasHardlinks && (
                                  <div
                                    style={{
                                      fontSize: 12,
                                      color: "var(--status-warn)",
                                    }}
                                  >
                                    {t.duplicates.hardlinkWarning}
                                  </div>
                                )}

                                <div
                                  style={{
                                    display: "flex",
                                    flexDirection: "column",
                                    gap: 8,
                                  }}
                                >
                                  {grp.copies.map((member) => {
                                    const isRetained =
                                      member.path === retainedPath;
                                    const isMarkedDelete =
                                      !isRetained &&
                                      selectedDeletes.includes(member.path);

                                    return (
                                      <div
                                        key={member.path}
                                        className={`dup-member-row ${
                                          isRetained
                                            ? "retained"
                                            : isMarkedDelete
                                            ? "marked-delete"
                                            : ""
                                        }`}
                                      >
                                        <div
                                          style={{
                                            display: "flex",
                                            flexDirection: "column",
                                            gap: 3,
                                            minWidth: 0,
                                            flex: 1,
                                          }}
                                        >
                                          <div
                                            style={{
                                              display: "flex",
                                              alignItems: "center",
                                              gap: 8,
                                              flexWrap: "wrap",
                                            }}
                                          >
                                            <strong>{member.file_name}</strong>
                                            {isRetained && (
                                              <span className="badge-ok">
                                                {t.duplicates.retainedBadge}
                                              </span>
                                            )}
                                            {isMarkedDelete && (
                                              <span
                                                className="badge-warn"
                                                style={{
                                                  background:
                                                    "rgba(239, 68, 68, 0.18)",
                                                  color: "var(--status-danger)",
                                                }}
                                              >
                                                {
                                                  t.duplicates
                                                    .markedForDeleteBadge
                                                }
                                              </span>
                                            )}
                                            {member.is_hardlink_alias && (
                                              <span className="badge-warn">
                                                Hardlink (
                                                {member.hardlink_count} links)
                                              </span>
                                            )}
                                          </div>
                                          <div
                                            className="mono-cell"
                                            style={{
                                              fontSize: 11.5,
                                              color: "var(--text-secondary)",
                                              overflow: "hidden",
                                              textOverflow: "ellipsis",
                                              whiteSpace: "nowrap",
                                            }}
                                          >
                                            {member.path}
                                          </div>
                                          <div
                                            className="mono-cell"
                                            style={{
                                              fontSize: 10.5,
                                              color: "var(--text-muted)",
                                            }}
                                          >
                                            {t.largeFiles.colModified}:{" "}
                                            {member.modified_at}
                                          </div>
                                        </div>

                                        <div
                                          style={{
                                            display: "flex",
                                            alignItems: "center",
                                            gap: 8,
                                            flexWrap: "wrap",
                                          }}
                                        >
                                          {!isRetained ? (
                                            <>
                                              <button
                                                type="button"
                                                className="icon-pill-btn"
                                                onClick={() =>
                                                  handleSetRetainedCopy(
                                                    grp.group_id,
                                                    member.path
                                                  )
                                                }
                                              >
                                                <ShieldCheck size={13} />
                                                <span>
                                                  {t.duplicates.keepThisCopy}
                                                </span>
                                              </button>
                                              <button
                                                type="button"
                                                className={
                                                  isMarkedDelete
                                                    ? "danger-pill-btn"
                                                    : "icon-pill-btn"
                                                }
                                                style={{
                                                  padding: "6px 12px",
                                                  fontSize: 12,
                                                }}
                                                onClick={() =>
                                                  handleToggleDeleteMember(
                                                    grp.group_id,
                                                    member.path
                                                  )
                                                }
                                              >
                                                <Trash2 size={13} />
                                                <span>
                                                  {
                                                    t.duplicates
                                                      .selectForRemoval
                                                  }
                                                </span>
                                              </button>
                                            </>
                                          ) : (
                                            <span
                                              style={{
                                                fontSize: 11.5,
                                                fontWeight: 700,
                                                color: "var(--status-ok)",
                                              }}
                                            >
                                              {t.duplicates.lockedRetainedHint}
                                            </span>
                                          )}

                                          <button
                                            type="button"
                                            className="icon-pill-btn"
                                            onClick={() =>
                                              void openPathInExplorer(
                                                member.path
                                              )
                                            }
                                            title={t.analyzer.openExplorer}
                                          >
                                            <ExternalLink size={12} />
                                          </button>
                                          <button
                                            type="button"
                                            className="icon-pill-btn"
                                            onClick={() =>
                                              void handleCopyPath(member.path)
                                            }
                                            title={t.analyzer.copyPath}
                                          >
                                            {copiedPath === member.path ? (
                                              <Check size={12} />
                                            ) : (
                                              <Copy size={12} />
                                            )}
                                          </button>
                                        </div>
                                      </div>
                                    );
                                  })}
                                </div>
                              </div>
                            );
                          })}
                        </div>
                      </div>
                    )}

                    {/* Floating Glossy Bottom Action Dock when duplicates are selected */}
                    {duplicateSelectionSummary.totalFilesSelected > 0 && (
                      <div className="floating-action-dock">
                        <div>
                          <div
                            style={{
                              fontSize: 15,
                              fontWeight: 800,
                              color: "var(--text-primary)",
                            }}
                          >
                            {duplicateSelectionSummary.totalFilesSelected}{" "}
                            {t.duplicates.floatingSelectedCount} •{" "}
                            <span
                              className="mono-cell"
                              style={{ color: "var(--accent-primary)" }}
                            >
                              {formatBytes(
                                duplicateSelectionSummary.totalBytesSelected
                              )}
                            </span>{" "}
                            {t.duplicates.floatingReclaimable}
                          </div>
                          {confirmingDupRemoval && (
                            <div
                              style={{
                                fontSize: 12,
                                color: "var(--status-warn)",
                                marginTop: 3,
                              }}
                            >
                              {t.duplicates.confirmModalWarning}
                            </div>
                          )}
                        </div>

                        <div style={{ display: "flex", gap: 10 }}>
                          {!confirmingDupRemoval ? (
                            <button
                              type="button"
                              className="primary-pill-btn"
                              onClick={() => setConfirmingDupRemoval(true)}
                            >
                              <Trash2 size={14} />
                              <span>{t.duplicates.reviewAndRemoveBtn}</span>
                            </button>
                          ) : (
                            <>
                              <button
                                type="button"
                                className="icon-pill-btn"
                                onClick={() => setConfirmingDupRemoval(false)}
                                disabled={removingDuplicates}
                              >
                                <span>{t.duplicates.cancelConfirmBtn}</span>
                              </button>
                              <button
                                type="button"
                                className="danger-pill-btn"
                                onClick={() =>
                                  void handleExecuteDuplicateRemoval()
                                }
                                disabled={removingDuplicates}
                              >
                                <Trash2 size={14} />
                                <span>{t.duplicates.confirmRemoveBtn}</span>
                              </button>
                            </>
                          )}
                        </div>
                      </div>
                    )}
                  </>
                )}
              </>
            )}

            {/* ============================================================
                VIEW 5: SAFE CLEANUP ENGINE (PHASE 3 — 9-STAGE PIPELINE)
               ============================================================ */}
            {activeNav === "cleanup" && (
              <>
                <div className="glass-panel">
                  <div className="panel-header-row">
                    <div>
                      <div className="status-pulse-line">
                        <span className="radar-pulse-wrap">
                          <span className="pulse-dot" />
                        </span>
                        <span>{t.cleanup.pipelineBadge}</span>
                      </div>
                      <h2 style={{ fontSize: 20, fontWeight: 800, marginTop: 4 }}>
                        {t.cleanup.title}
                      </h2>
                      <p style={{ fontSize: 13, color: "var(--text-secondary)" }}>
                        {t.cleanup.subtitle}
                      </p>
                    </div>

                    {activeCleanupJobId ? (
                      <button
                        type="button"
                        className="danger-pill-btn"
                        onClick={() => void handleCancelCleanupScan()}
                      >
                        <Square size={14} />
                        <span>{t.cleanup.cancelBtn}</span>
                      </button>
                    ) : (
                      <button
                        type="button"
                        className="primary-pill-btn"
                        onClick={() => void triggerSafeCleanupScan()}
                      >
                        <Sparkles size={14} />
                        <span>{t.cleanup.scanBtn}</span>
                      </button>
                    )}
                  </div>

                  {/* 9-Stage Visual Pipeline Bar */}
                  <div className="pipeline-steps-bar">
                    {[
                      "1. Scan",
                      "2. Analyze",
                      "3. Plan",
                      "4. Preview",
                      "5. Confirm",
                      "6. Validate",
                      "7. Execute",
                      "8. Verify",
                      "9. Report",
                    ].map((stepLabel, idx) => {
                      const isDone =
                        cleanupReport !== null ||
                        (cleanupPlan !== null && idx <= 3);
                      const isActive =
                        (activeCleanupJobId !== null && idx <= 2) ||
                        (cleanupPlan !== null && !cleanupReport && idx === 3) ||
                        (confirmingCleanup && idx >= 4 && idx <= 5);
                      return (
                        <span
                          key={stepLabel}
                          className={`pipeline-step-pill ${
                            isDone ? "done" : isActive ? "active" : ""
                          }`}
                        >
                          {isDone && <Check size={11} />}
                          <span>{stepLabel}</span>
                        </span>
                      );
                    })}
                  </div>
                </div>

                {/* Live Cleanup Scan Banner with Animated Liquid Wave Bar */}
                {activeCleanupJobId && (
                  <div className="glass-panel">
                    <div className="panel-header-row">
                      <div className="status-pulse-line">
                        <span className="pulse-dot" />
                        <span>{t.cleanup.scanningTitle}</span>
                      </div>
                      {cleanupProgress && (
                        <span className="mono-cell">
                          {cleanupProgress.files_processed.toLocaleString()}{" "}
                          {t.analyzer.filesSeen} •{" "}
                          {formatBytes(cleanupProgress.bytes_processed)}
                        </span>
                      )}
                    </div>
                    <div className="animated-wave-bar">
                      <div className="animated-wave-fill" />
                    </div>
                    {cleanupProgress && (
                      <div
                        className="mono-cell"
                        style={{
                          color: "var(--text-secondary)",
                          overflow: "hidden",
                          textOverflow: "ellipsis",
                          whiteSpace: "nowrap",
                        }}
                      >
                        {cleanupProgress.current_path}
                      </div>
                    )}
                  </div>
                )}

                {/* Cleanup Execution Report Banner */}
                {cleanupReport && (
                  <div
                    className="glass-panel"
                    style={{
                      borderColor: "var(--status-ok)",
                      background: "rgba(16, 185, 129, 0.12)",
                    }}
                  >
                    <div
                      style={{
                        display: "flex",
                        alignItems: "center",
                        gap: 10,
                      }}
                    >
                      <ShieldCheck size={18} color="var(--status-ok)" />
                      <strong>
                        {t.cleanup.cleanSuccessBanner}{" "}
                        {cleanupReport.succeeded_count} verified removed •{" "}
                        {formatBytes(cleanupReport.bytes_reclaimed)} reclaimed
                        {cleanupReport.skipped_or_failed_count > 0
                          ? ` (${cleanupReport.skipped_or_failed_count} locked/skipped)`
                          : ""}
                      </strong>
                    </div>
                  </div>
                )}

                {/* Declarative Cleanup Rules Grid */}
                <div className="glass-panel">
                  <div className="panel-title">
                    <ShieldCheck size={18} color="var(--accent-primary)" />
                    <span>{t.cleanup.rulesSectionTitle}</span>
                  </div>

                  <div className="cleanup-rules-grid">
                    {cleanupRules.map((rule) => {
                      const isRuleEnabled = enabledRuleIds.includes(rule.id);
                      const summary = cleanupPlan?.rule_summaries.find(
                        (s) => s.rule_id === rule.id
                      );
                      return (
                        <div
                          key={rule.id}
                          className={`cleanup-rule-card ${
                            isRuleEnabled ? "selected" : ""
                          }`}
                          onClick={() => handleToggleRuleEnabled(rule.id)}
                        >
                          <div
                            style={{
                              display: "flex",
                              alignItems: "center",
                              justifyContent: "space-between",
                              gap: 8,
                            }}
                          >
                            <div
                              style={{
                                display: "flex",
                                alignItems: "center",
                                gap: 8,
                              }}
                            >
                              <input
                                type="checkbox"
                                checked={isRuleEnabled}
                                onChange={() =>
                                  handleToggleRuleEnabled(rule.id)
                                }
                                onClick={(e) => e.stopPropagation()}
                                style={{
                                  width: 16,
                                  height: 16,
                                  cursor: "pointer",
                                }}
                              />
                              <strong>
                                {lang === "ar" ? rule.name_ar : rule.name_en}
                              </strong>
                            </div>
                            <span
                              className={
                                rule.risk_level === "Low"
                                  ? "badge-ok"
                                  : "badge-warn"
                              }
                            >
                              {rule.risk_level === "Low"
                                ? t.cleanup.riskLow
                                : t.cleanup.riskMedium}
                            </span>
                          </div>

                          <p
                            style={{
                              fontSize: 12,
                              color: "var(--text-secondary)",
                            }}
                          >
                            {lang === "ar" ? rule.reason_ar : rule.reason_en}
                          </p>

                          {summary && (
                            <div
                              className="mono-cell"
                              style={{
                                display: "flex",
                                justifyContent: "space-between",
                                fontSize: 11.5,
                                paddingTop: 6,
                                borderTop: "1px solid var(--glass-border)",
                              }}
                            >
                              <span>
                                {summary.candidates_count} items (
                                {summary.locked_skipped_count} locked)
                              </span>
                              <strong style={{ color: "var(--accent-primary)" }}>
                                {formatBytes(summary.total_logical_bytes)}
                              </strong>
                            </div>
                          )}
                        </div>
                      );
                    })}
                  </div>
                </div>

                {!cleanupPlan && !activeCleanupJobId && (
                  <div
                    className="glass-panel"
                    style={{ textAlign: "center", paddingBlock: 42 }}
                  >
                    <Sparkles
                      size={38}
                      color="var(--accent-primary)"
                      style={{ marginInline: "auto" }}
                    />
                    <h3 style={{ marginTop: 8 }}>
                      {t.cleanup.emptyStateTitle}
                    </h3>
                    <p style={{ color: "var(--text-secondary)" }}>
                      {t.cleanup.emptyStateDesc}
                    </p>
                  </div>
                )}

                {cleanupPlan && (
                  <>
                    {/* Summary Metrics Strip */}
                    <div
                      style={{
                        display: "grid",
                        gridTemplateColumns:
                          "repeat(auto-fit, minmax(195px, 1fr))",
                        gap: 14,
                      }}
                    >
                      <div className="glass-panel" style={{ padding: 16 }}>
                        <span
                          style={{
                            fontSize: 12,
                            color: "var(--text-secondary)",
                          }}
                        >
                          {t.cleanup.totalCandidates}
                        </span>
                        <strong
                          className="mono-cell"
                          style={{
                            fontSize: 22,
                            color: "var(--accent-primary)",
                          }}
                        >
                          {cleanupPlan.total_candidates.toLocaleString()}
                        </strong>
                      </div>
                      <div className="glass-panel" style={{ padding: 16 }}>
                        <span
                          style={{
                            fontSize: 12,
                            color: "var(--text-secondary)",
                          }}
                        >
                          {t.cleanup.reclaimableBytes}
                        </span>
                        <strong
                          className="mono-cell"
                          style={{ fontSize: 22, color: "var(--status-ok)" }}
                        >
                          {formatBytes(cleanupPlan.total_reclaimable_bytes)}
                        </strong>
                      </div>
                      <div className="glass-panel" style={{ padding: 16 }}>
                        <span
                          style={{
                            fontSize: 12,
                            color: "var(--text-secondary)",
                          }}
                        >
                          {t.cleanup.lockedSkipped}
                        </span>
                        <strong className="mono-cell" style={{ fontSize: 20 }}>
                          {cleanupPlan.total_locked_skipped.toLocaleString()} (
                          {cleanupPlan.duration_ms} ms)
                        </strong>
                      </div>
                    </div>

                    {cleanupPlan.candidates.length === 0 ? (
                      <div
                        className="glass-panel"
                        style={{ textAlign: "center", paddingBlock: 38 }}
                      >
                        <ShieldCheck
                          size={36}
                          color="var(--status-ok)"
                          style={{ marginInline: "auto" }}
                        />
                        <h3 style={{ marginTop: 8 }}>
                          {t.cleanup.zeroCandidatesTitle}
                        </h3>
                        <p style={{ color: "var(--text-secondary)" }}>
                          {t.cleanup.zeroCandidatesDesc}
                        </p>
                      </div>
                    ) : (
                      <div className="glass-panel">
                        <div className="panel-header-row">
                          <div className="panel-title">
                            <Sparkles size={18} color="var(--accent-primary)" />
                            <span>{t.cleanup.candidatesSectionTitle}</span>
                          </div>

                          <div style={{ display: "flex", gap: 8 }}>
                            <button
                              type="button"
                              className="icon-pill-btn"
                              onClick={handleSelectAllSafeCleanup}
                            >
                              <Check size={13} />
                              <span>{t.cleanup.selectAllSafe}</span>
                            </button>
                            <button
                              type="button"
                              className="icon-pill-btn"
                              onClick={() => setSelectedCleanupPaths([])}
                            >
                              <span>{t.cleanup.clearSelection}</span>
                            </button>
                          </div>
                        </div>

                        <table className="data-table">
                          <thead>
                            <tr>
                              <th style={{ width: 52 }}>
                                {t.largeFiles.colSelect}
                              </th>
                              <th>{t.largeFiles.colName}</th>
                              <th>{t.largeFiles.colCategory}</th>
                              <th>{t.largeFiles.colLogicalSize}</th>
                              <th>{t.largeFiles.colActions}</th>
                            </tr>
                          </thead>
                          <tbody>
                            {cleanupPlan.candidates.slice(0, 150).map((cand) => {
                              const isSelected = selectedCleanupPaths.includes(
                                cand.path
                              );
                              return (
                                <tr key={cand.candidate_id}>
                                  <td>
                                    <input
                                      type="checkbox"
                                      checked={isSelected}
                                      onChange={() =>
                                        handleToggleCleanupCandidate(cand.path)
                                      }
                                      style={{
                                        width: 16,
                                        height: 16,
                                        cursor: "pointer",
                                      }}
                                    />
                                  </td>
                                  <td>
                                    <div style={{ fontWeight: 700 }}>
                                      {cand.file_name}
                                    </div>
                                    <div
                                      className="mono-cell"
                                      style={{
                                        color: "var(--text-secondary)",
                                        fontSize: 11,
                                      }}
                                    >
                                      {cand.path}
                                    </div>
                                    <div
                                      style={{
                                        fontSize: 11,
                                        color: "var(--text-muted)",
                                      }}
                                    >
                                      {lang === "ar"
                                        ? cand.reason_ar
                                        : cand.reason_en}
                                    </div>
                                  </td>
                                  <td>
                                    <span
                                      className={
                                        cand.risk_level === "Low"
                                          ? "badge-ok"
                                          : "badge-warn"
                                      }
                                    >
                                      {cand.category}
                                    </span>
                                  </td>
                                  <td
                                    className="mono-cell"
                                    style={{
                                      fontWeight: 800,
                                      color: "var(--accent-primary)",
                                    }}
                                  >
                                    {formatBytes(cand.logical_size)}
                                  </td>
                                  <td>
                                    <div style={{ display: "flex", gap: 6 }}>
                                      <button
                                        type="button"
                                        className="icon-pill-btn"
                                        onClick={() =>
                                          void openPathInExplorer(cand.path)
                                        }
                                        title={t.analyzer.openExplorer}
                                      >
                                        <ExternalLink size={12} />
                                      </button>
                                      <button
                                        type="button"
                                        className="icon-pill-btn"
                                        onClick={() =>
                                          void handleCopyPath(cand.path)
                                        }
                                        title={t.analyzer.copyPath}
                                      >
                                        {copiedPath === cand.path ? (
                                          <Check size={12} />
                                        ) : (
                                          <Copy size={12} />
                                        )}
                                      </button>
                                    </div>
                                  </td>
                                </tr>
                              );
                            })}
                          </tbody>
                        </table>
                      </div>
                    )}

                    {cleanupSelectionSummary.count > 0 && (
                      <div className="floating-action-dock">
                        <div>
                          <div
                            style={{
                              fontSize: 15,
                              fontWeight: 800,
                              color: "var(--text-primary)",
                            }}
                          >
                            {cleanupSelectionSummary.count}{" "}
                            {t.cleanup.floatingSelectedCount} •{" "}
                            <span
                              className="mono-cell"
                              style={{ color: "var(--accent-primary)" }}
                            >
                              {formatBytes(cleanupSelectionSummary.bytes)}
                            </span>{" "}
                            {t.cleanup.floatingReclaimable}
                          </div>
                          {confirmingCleanup && (
                            <div
                              style={{
                                fontSize: 12,
                                color: "var(--status-warn)",
                                marginTop: 3,
                              }}
                            >
                              {t.cleanup.confirmWarning}
                            </div>
                          )}
                        </div>

                        <div style={{ display: "flex", gap: 10 }}>
                          {!confirmingCleanup ? (
                            <button
                              type="button"
                              className="primary-pill-btn"
                              onClick={() => setConfirmingCleanup(true)}
                            >
                              <Sparkles size={14} />
                              <span>{t.cleanup.reviewCleanBtn}</span>
                            </button>
                          ) : (
                            <>
                              <button
                                type="button"
                                className="icon-pill-btn"
                                onClick={() => setConfirmingCleanup(false)}
                                disabled={executingCleanup}
                              >
                                <span>{t.cleanup.cancelConfirmBtn}</span>
                              </button>
                              <button
                                type="button"
                                className="danger-pill-btn"
                                onClick={() => void handleExecuteSafeCleanup()}
                                disabled={executingCleanup}
                              >
                                <Trash2 size={14} />
                                <span>{t.cleanup.confirmCleanBtn}</span>
                              </button>
                            </>
                          )}
                        </div>
                      </div>
                    )}
                  </>
                )}
              </>
            )}

            {/* ============================================================
                VIEW 6: APPLICATIONS, GAMES & ORPHANED LEFTOVERS (PHASE 4)
               ============================================================ */}
            {activeNav === "applications" && (
              <>
                <div className="glass-panel">
                  <div className="panel-header-row">
                    <div>
                      <h2 style={{ fontSize: 20, fontWeight: 800 }}>
                        {t.applications.title}
                      </h2>
                      <p style={{ fontSize: 13, color: "var(--text-secondary)" }}>
                        {t.applications.subtitle}
                      </p>
                    </div>

                    <div style={{ display: "flex", gap: 10, alignItems: "center" }}>
                      {activeInventoryJobId ? (
                        <button
                          type="button"
                          className="danger-pill-btn"
                          onClick={() => void handleCancelInventoryScan()}
                        >
                          <Square size={14} />
                          <span>{t.applications.cancelBtn}</span>
                        </button>
                      ) : (
                        <button
                          type="button"
                          className="primary-pill-btn"
                          onClick={() => void handleStartInventoryScan()}
                        >
                          <Package size={15} />
                          <span>{t.applications.scanBtn}</span>
                        </button>
                      )}
                    </div>
                  </div>

                  {/* Sub-Tabs Switcher (Crystal Glass) */}
                  <div
                    style={{
                      display: "flex",
                      justifyContent: "space-between",
                      alignItems: "center",
                      flexWrap: "wrap",
                      gap: 12,
                    }}
                  >
                    <div className="glass-subtabs">
                      <button
                        type="button"
                        className={`glass-subtab-btn ${
                          inventoryTab === "leftovers" ? "active" : ""
                        }`}
                        onClick={() => setInventoryTab("leftovers")}
                      >
                        <Layers size={15} />
                        <span>{t.applications.tabLeftovers}</span>
                        {inventoryResult && (
                          <span className="badge-warn">
                            {inventoryResult.total_leftovers_count}
                          </span>
                        )}
                      </button>

                      <button
                        type="button"
                        className={`glass-subtab-btn ${
                          inventoryTab === "installed" ? "active" : ""
                        }`}
                        onClick={() => setInventoryTab("installed")}
                      >
                        <Package size={15} />
                        <span>{t.applications.tabInstalled}</span>
                        {inventoryResult && (
                          <span className="badge-ok">
                            {inventoryResult.installed_apps.length}
                          </span>
                        )}
                      </button>
                    </div>

                    {inventoryResult && (
                      <div
                        style={{
                          display: "flex",
                          gap: 14,
                          flexWrap: "wrap",
                          fontSize: 12.5,
                          color: "var(--text-secondary)",
                        }}
                      >
                        <span>
                          <strong>{inventoryResult.total_apps_count}</strong>{" "}
                          {t.applications.totalApps}
                        </span>
                        <span>•</span>
                        <span>
                          <strong>{inventoryResult.total_games_count}</strong>{" "}
                          {t.applications.totalGames}
                        </span>
                        <span>•</span>
                        <span>
                          <strong style={{ color: "var(--status-ok)" }}>
                            {formatBytes(
                              inventoryResult.high_confidence_reclaimable_bytes
                            )}
                          </strong>{" "}
                          {t.applications.highConfidenceBytes}
                        </span>
                      </div>
                    )}
                  </div>
                </div>

                {/* Live Inventory Scan Progress */}
                {activeInventoryJobId && inventoryProgress && (
                  <div className="glass-panel">
                    <div className="panel-title">
                      <span className="radar-pulse-wrap">
                        <Activity size={14} color="var(--status-ok)" />
                      </span>
                      <span>{t.applications.scanningTitle}</span>
                    </div>
                    <div className="animated-wave-bar">
                      <div className="animated-wave-fill" />
                    </div>
                    <div
                      className="mono-cell"
                      style={{ fontSize: 12, color: "var(--text-secondary)" }}
                    >
                      {inventoryProgress.current_path}
                    </div>
                  </div>
                )}

                {/* Leftover Removal Success Banner */}
                {leftoverRemovalReport && (
                  <div
                    className="glass-panel"
                    style={{
                      borderColor: "rgba(16, 185, 129, 0.45)",
                      background:
                        "linear-gradient(135deg, rgba(16, 185, 129, 0.14), rgba(14, 165, 233, 0.06))",
                    }}
                  >
                    <div className="panel-title" style={{ color: "var(--status-ok)" }}>
                      <ShieldCheck size={18} />
                      <span>
                        {t.applications.removeSuccessBanner}{" "}
                        {leftoverRemovalReport.succeeded_count} /{" "}
                        {leftoverRemovalReport.total_requested} (
                        {formatBytes(leftoverRemovalReport.bytes_reclaimed)})
                      </span>
                    </div>
                  </div>
                )}

                {/* Empty State before first scan */}
                {!inventoryResult && !activeInventoryJobId && (
                  <div
                    className="glass-panel"
                    style={{ textAlign: "center", padding: "42px 24px" }}
                  >
                    <Package
                      size={40}
                      color="var(--accent-primary)"
                      style={{ margin: "0 auto" }}
                    />
                    <h3 style={{ fontSize: 17, fontWeight: 800 }}>
                      {t.applications.emptyStateTitle}
                    </h3>
                    <p
                      style={{
                        fontSize: 13,
                        color: "var(--text-secondary)",
                        maxWidth: 620,
                        margin: "0 auto",
                      }}
                    >
                      {t.applications.emptyStateDesc}
                    </p>
                  </div>
                )}

                {/* SUB-TAB 1: ORPHANED LEFTOVERS */}
                {inventoryResult && inventoryTab === "leftovers" && (
                  <>
                    <div className="glass-panel">
                      <div
                        className="panel-header-row"
                        style={{ gap: 10, justifyContent: "space-between" }}
                      >
                        <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
                          {(
                            [
                              { id: "ALL", label: t.applications.filterAll },
                              { id: "High", label: t.applications.filterHigh },
                              { id: "Medium", label: t.applications.filterMedium },
                              { id: "Low", label: t.applications.filterLow },
                              {
                                id: "SAVED",
                                label: t.applications.filterSavedProtected,
                              },
                            ] as const
                          ).map((f) => (
                            <button
                              key={f.id}
                              type="button"
                              className={`preset-chip ${
                                leftoverFilter === f.id ? "active" : ""
                              }`}
                              onClick={() => setLeftoverFilter(f.id)}
                            >
                              <span>{f.label}</span>
                            </button>
                          ))}
                        </div>

                        <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
                          <button
                            type="button"
                            className="icon-pill-btn"
                            onClick={handleSelectHighConfidenceLeftovers}
                          >
                            <Sparkles size={13} />
                            <span>{t.applications.selectHighConfidence}</span>
                          </button>
                          {selectedLeftoverPaths.length > 0 && (
                            <button
                              type="button"
                              className="icon-pill-btn"
                              onClick={() => {
                                setSelectedLeftoverPaths([]);
                                setConfirmingLeftoverRemoval(false);
                              }}
                            >
                              <span>{t.applications.clearSelection}</span>
                            </button>
                          )}
                        </div>
                      </div>

                      <input
                        type="text"
                        className="glass-input"
                        value={leftoverSearch}
                        onChange={(e) => setLeftoverSearch(e.target.value)}
                        placeholder={t.applications.searchLeftoversPlaceholder}
                      />

                      {filteredLeftovers.length === 0 ? (
                        <div style={{ textAlign: "center", padding: "28px 16px" }}>
                          <ShieldCheck
                            size={32}
                            color="var(--status-ok)"
                            style={{ margin: "0 auto 8px" }}
                          />
                          <div style={{ fontWeight: 800, fontSize: 15 }}>
                            {t.applications.zeroLeftoversTitle}
                          </div>
                          <div
                            style={{
                              fontSize: 12.5,
                              color: "var(--text-secondary)",
                              marginTop: 4,
                            }}
                          >
                            {t.applications.zeroLeftoversDesc}
                          </div>
                        </div>
                      ) : (
                        <div className="leftover-card-list">
                          {filteredLeftovers.map((item) => {
                            const isSelected = selectedLeftoverPaths.includes(
                              item.path
                            );
                            const confClass =
                              item.confidence === "High"
                                ? "high"
                                : item.confidence === "Medium"
                                ? "medium"
                                : "low";
                            const confLabel =
                              item.confidence === "High"
                                ? t.applications.confidenceHigh
                                : item.confidence === "Medium"
                                ? t.applications.confidenceMedium
                                : t.applications.confidenceLow;

                            return (
                              <div
                                key={item.id}
                                className={`leftover-glass-card ${
                                  isSelected ? "selected" : ""
                                }`}
                              >
                                <div
                                  style={{
                                    display: "flex",
                                    alignItems: "flex-start",
                                    gap: 14,
                                    flex: 1,
                                  }}
                                >
                                  <input
                                    type="checkbox"
                                    checked={isSelected}
                                    disabled={item.has_locked_files}
                                    onChange={() =>
                                      handleToggleLeftoverSelect(item.path)
                                    }
                                    style={{
                                      width: 18,
                                      height: 18,
                                      marginTop: 4,
                                      cursor: item.has_locked_files
                                        ? "not-allowed"
                                        : "pointer",
                                      accentColor: "var(--accent-primary)",
                                    }}
                                  />

                                  <div
                                    style={{
                                      display: "flex",
                                      flexDirection: "column",
                                      gap: 6,
                                      flex: 1,
                                    }}
                                  >
                                    <div
                                      style={{
                                        display: "flex",
                                        alignItems: "center",
                                        gap: 8,
                                        flexWrap: "wrap",
                                      }}
                                    >
                                      <span
                                        style={{
                                          fontSize: 15,
                                          fontWeight: 800,
                                          color: "var(--text-primary)",
                                        }}
                                      >
                                        {item.folder_name}
                                      </span>

                                      <span
                                        className={`confidence-badge ${confClass}`}
                                      >
                                        {confLabel}
                                      </span>

                                      <span className="preset-chip">
                                        {item.scope_label}
                                      </span>

                                      {item.contains_saved_games_or_user_data && (
                                        <span className="saved-shield-badge">
                                          <ShieldAlert size={12} />
                                          <span>
                                            {t.applications.savedDataProtectedBadge}
                                          </span>
                                        </span>
                                      )}

                                      {item.has_locked_files && (
                                        <span className="badge-warn">
                                          {t.applications.lockedFilesBadge}
                                        </span>
                                      )}
                                    </div>

                                    <div
                                      className="mono-cell"
                                      style={{
                                        fontSize: 11.5,
                                        color: "var(--text-muted)",
                                      }}
                                    >
                                      {item.path}
                                    </div>

                                    <div
                                      style={{
                                        fontSize: 12.5,
                                        color: "var(--text-secondary)",
                                      }}
                                    >
                                      {lang === "ar"
                                        ? item.reason_ar
                                        : item.reason_en}
                                    </div>
                                  </div>
                                </div>

                                <div
                                  style={{
                                    display: "flex",
                                    flexDirection: "column",
                                    alignItems: "flex-end",
                                    gap: 8,
                                    minWidth: 140,
                                  }}
                                >
                                  <div
                                    className="mono-cell"
                                    style={{
                                      fontSize: 15,
                                      fontWeight: 800,
                                      color: "var(--accent-primary)",
                                    }}
                                  >
                                    {formatBytes(item.logical_size_bytes)}
                                  </div>
                                  <div
                                    style={{
                                      fontSize: 11.5,
                                      color: "var(--text-muted)",
                                    }}
                                  >
                                    {item.file_count}{" "}
                                    {lang === "ar" ? "ملف" : "files"} •{" "}
                                    {item.age_days}{" "}
                                    {lang === "ar" ? "يوم" : "d ago"}
                                  </div>
                                  <div style={{ display: "flex", gap: 6 }}>
                                    <button
                                      type="button"
                                      className="icon-pill-btn"
                                      onClick={() =>
                                        void openPathInExplorer(item.path)
                                      }
                                      title={t.analyzer.openExplorer}
                                    >
                                      <ExternalLink size={12} />
                                    </button>
                                    <button
                                      type="button"
                                      className="icon-pill-btn"
                                      onClick={() =>
                                        void handleCopyPath(item.path)
                                      }
                                      title={t.analyzer.copyPath}
                                    >
                                      {copiedPath === item.path ? (
                                        <Check size={12} />
                                      ) : (
                                        <Copy size={12} />
                                      )}
                                    </button>
                                  </div>
                                </div>
                              </div>
                            );
                          })}
                        </div>
                      )}
                    </div>

                    {leftoverSelectionSummary.count > 0 && (
                      <div className="floating-action-dock">
                        <div>
                          <div
                            style={{
                              fontSize: 15,
                              fontWeight: 800,
                              color: "var(--text-primary)",
                            }}
                          >
                            {leftoverSelectionSummary.count}{" "}
                            {t.applications.floatingSelectedCount} •{" "}
                            <span
                              className="mono-cell"
                              style={{ color: "var(--accent-primary)" }}
                            >
                              {formatBytes(leftoverSelectionSummary.bytes)}
                            </span>{" "}
                            {t.applications.floatingReclaimable}
                          </div>
                          {confirmingLeftoverRemoval && (
                            <div
                              style={{
                                fontSize: 12,
                                color: "var(--status-warn)",
                                marginTop: 3,
                              }}
                            >
                              {t.applications.confirmWarning}
                            </div>
                          )}
                        </div>

                        <div style={{ display: "flex", gap: 10 }}>
                          {!confirmingLeftoverRemoval ? (
                            <button
                              type="button"
                              className="primary-pill-btn"
                              onClick={() => setConfirmingLeftoverRemoval(true)}
                            >
                              <Trash2 size={14} />
                              <span>{t.applications.reviewRemoveBtn}</span>
                            </button>
                          ) : (
                            <>
                              <button
                                type="button"
                                className="icon-pill-btn"
                                onClick={() =>
                                  setConfirmingLeftoverRemoval(false)
                                }
                                disabled={removingLeftovers}
                              >
                                <span>{t.applications.cancelConfirmBtn}</span>
                              </button>
                              <button
                                type="button"
                                className="danger-pill-btn"
                                onClick={() =>
                                  void handleExecuteLeftoverRemoval()
                                }
                                disabled={removingLeftovers}
                              >
                                <Trash2 size={14} />
                                <span>{t.applications.confirmRemoveBtn}</span>
                              </button>
                            </>
                          )}
                        </div>
                      </div>
                    )}
                  </>
                )}

                {/* SUB-TAB 2: INSTALLED APPS & GAMES INVENTORY */}
                {inventoryResult && inventoryTab === "installed" && (
                  <div className="glass-panel">
                    <div
                      className="panel-header-row"
                      style={{ gap: 10, justifyContent: "space-between" }}
                    >
                      <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
                        {[
                          { id: "ALL", label: t.applications.sourceAll },
                          {
                            id: "Windows Registry",
                            label: t.applications.sourceRegistry,
                          },
                          { id: "Steam", label: t.applications.sourceSteam },
                          {
                            id: "Epic Games",
                            label: t.applications.sourceEpic,
                          },
                        ].map((src) => (
                          <button
                            key={src.id}
                            type="button"
                            className={`preset-chip ${
                              appSourceFilter === src.id ? "active" : ""
                            }`}
                            onClick={() => setAppSourceFilter(src.id)}
                          >
                            <span>{src.label}</span>
                          </button>
                        ))}
                      </div>
                    </div>

                    <input
                      type="text"
                      className="glass-input"
                      value={appSearch}
                      onChange={(e) => setAppSearch(e.target.value)}
                      placeholder={t.applications.searchAppsPlaceholder}
                    />

                    {uninstallerNotice && (
                      <div
                        style={{
                          padding: "12px 16px",
                          borderRadius: 14,
                          border: "1px solid rgba(14, 165, 233, 0.45)",
                          background: "rgba(14, 165, 233, 0.14)",
                          display: "flex",
                          alignItems: "center",
                          justifyContent: "space-between",
                          gap: 12,
                          flexWrap: "wrap",
                        }}
                      >
                        <span style={{ fontSize: 13, fontWeight: 700 }}>
                          {uninstallerNotice}
                        </span>
                        <button
                          type="button"
                          className="primary-pill-btn"
                          onClick={() => {
                            setInventoryTab("leftovers");
                            void handleStartInventoryScan();
                          }}
                        >
                          <RefreshCw size={13} />
                          <span>
                            {lang === "ar"
                              ? "إعادة فحص البقايا الآن"
                              : "Re-scan Leftovers Now"}
                          </span>
                        </button>
                      </div>
                    )}

                    <div className="table-scroll-container">
                      <table className="data-table">
                        <thead>
                          <tr>
                            <th>{t.applications.colAppName}</th>
                            <th>{t.applications.colPublisher}</th>
                            <th>{t.applications.colVersion}</th>
                            <th>{t.applications.colSource}</th>
                            <th>{t.applications.colSize}</th>
                            <th>{t.applications.colActions}</th>
                          </tr>
                        </thead>
                        <tbody>
                          {filteredInstalledApps.map((app) => (
                            <tr key={app.id}>
                              <td>
                                <div style={{ fontWeight: 800 }}>
                                  {app.display_name}
                                </div>
                                {app.install_location && (
                                  <div
                                    className="mono-cell"
                                    style={{
                                      fontSize: 11,
                                      color: "var(--text-muted)",
                                    }}
                                  >
                                    {app.install_location}
                                  </div>
                                )}
                              </td>
                              <td>
                                <div>{app.publisher}</div>
                                <div
                                  style={{
                                    fontSize: 11,
                                    color: "var(--text-muted)",
                                  }}
                                >
                                  {app.scope}
                                </div>
                              </td>
                              <td className="mono-cell">
                                {app.display_version}
                              </td>
                              <td>
                                <span
                                  className={
                                    app.source === "Steam" ||
                                    app.source === "Epic Games"
                                      ? "confidence-badge medium"
                                      : "badge-ok"
                                  }
                                >
                                  {app.source}
                                </span>
                              </td>
                              <td>
                                {app.size_bytes > 0 ? (
                                  <div>
                                    <div
                                      className="mono-cell"
                                      style={{
                                        fontWeight: 800,
                                        color: "var(--accent-primary)",
                                      }}
                                    >
                                      {formatBytes(app.size_bytes)}
                                    </div>
                                    <div
                                      style={{
                                        fontSize: 10.5,
                                        color: "var(--text-muted)",
                                      }}
                                    >
                                      {app.is_size_estimated
                                        ? t.applications.estimatedSizeBadge
                                        : t.applications.exactSizeBadge}
                                    </div>
                                  </div>
                                ) : (
                                  <span style={{ color: "var(--text-muted)" }}>
                                    —
                                  </span>
                                )}
                              </td>
                              <td>
                                <div
                                  style={{
                                    display: "flex",
                                    alignItems: "center",
                                    gap: 8,
                                    flexWrap: "wrap",
                                  }}
                                >
                                  <button
                                    type="button"
                                    className="uninstall-pill-btn"
                                    onClick={() =>
                                      void handleLaunchAppUninstaller(
                                        app.display_name,
                                        app.source,
                                        app.uninstall_string
                                      )
                                    }
                                    title={
                                      lang === "ar"
                                        ? "إلغاء تثبيت التطبيق أو اللعبة"
                                        : "Uninstall App / Game"
                                    }
                                  >
                                    <Trash2 size={12} />
                                    <span>
                                      {lang === "ar"
                                        ? "إلغاء التثبيت"
                                        : "Uninstall"}
                                    </span>
                                  </button>

                                  {app.install_location && (
                                    <button
                                      type="button"
                                      className="icon-pill-btn"
                                      onClick={() =>
                                        void openPathInExplorer(
                                          app.install_location!
                                        )
                                      }
                                      title={t.analyzer.openExplorer}
                                    >
                                      <FolderOpen size={12} />
                                    </button>
                                  )}
                                </div>
                              </td>
                            </tr>
                          ))}
                        </tbody>
                      </table>
                    </div>
                  </div>
                )}
              </>
            )}

            {/* ============================================================
                VIEW 6.5: STARTUP MANAGER & SYSTEM BOOSTER (NEW FEATURE)
               ============================================================ */}
            {activeNav === "startup" && (
              <>
                <div className="glass-panel">
                  <div className="panel-header-row">
                    <div>
                      <div className="panel-title">
                        <Cpu size={20} color="var(--accent-primary)" />
                        <span>
                          {lang === "ar"
                            ? "مسرّع النظام ومدير برامج بدء التشغيل (System Booster & Startup)"
                            : "System Booster & Windows Startup Manager"}
                        </span>
                      </div>
                      <p
                        style={{
                          fontSize: 13,
                          color: "var(--text-secondary)",
                          marginTop: 4,
                        }}
                      >
                        {lang === "ar"
                          ? "مراقبة حية للذاكرة العشوائية (RAM)، تسريع استجابة الشبكة ومصغرات النظام، وإدارة البرامج التي تعمل تلقائياً عند إقلاع Windows."
                          : "Live RAM telemetry, instant DNS & thumbnail cache acceleration, and real-time management of Windows startup entries."}
                      </p>
                    </div>
                    <button
                      type="button"
                      className="primary-pill-btn"
                      onClick={async () => {
                        const snap = await fetchStartupAndBoosterStatus();
                        setStartupSnapshot(snap);
                      }}
                    >
                      <RefreshCw size={14} />
                      <span>
                        {lang === "ar" ? "تحديث القراءات الحية" : "Refresh Telemetry"}
                      </span>
                    </button>
                  </div>

                  {startupSnapshot && (
                    <div className="bento-cards-row">
                      <div className="vibrant-card emerald">
                        <div>
                          <div className="vibrant-card-head">
                            <div className="vibrant-icon-box">
                              <Cpu size={17} />
                            </div>
                            <span>
                              {lang === "ar"
                                ? "استهلاك الذاكرة العشوائية (RAM)"
                                : "Physical RAM Load"}
                            </span>
                          </div>
                          <div className="vibrant-metric">
                            {startupSnapshot.memory.memory_load_percent}%
                            <span>
                              {formatBytes(startupSnapshot.memory.used_physical_bytes)} /{" "}
                              {formatBytes(startupSnapshot.memory.total_physical_bytes)}
                            </span>
                          </div>
                          <p className="vibrant-sub">
                            {lang === "ar"
                              ? `الذاكرة المتاحة حالياً: ${formatBytes(
                                  startupSnapshot.memory.avail_physical_bytes
                                )}`
                              : `Available RAM: ${formatBytes(
                                  startupSnapshot.memory.avail_physical_bytes
                                )}`}
                          </p>
                        </div>
                        <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
                          <button
                            type="button"
                            className="vibrant-pill-btn"
                            disabled={cleaningMemory}
                            onClick={() => void handleExecuteMemoryReduction("Deep", "booster_card")}
                          >
                            {cleaningMemory
                              ? lang === "ar"
                                ? "جاري التفريغ..."
                                : "Cleaning..."
                              : lang === "ar"
                              ? "تفريغ عميق للرامات الآن ⚡"
                              : "Deep Clean RAM Now ⚡"}
                          </button>
                          <button
                            type="button"
                            className="vibrant-pill-btn"
                            onClick={() => setActiveNav("memory")}
                          >
                            {lang === "ar"
                              ? "فتح مركز الرامات المستقل ←"
                              : "Open Memory Engine →"}
                          </button>
                        </div>
                      </div>

                      <div className="vibrant-card ocean">
                        <div>
                          <div className="vibrant-card-head">
                            <div className="vibrant-icon-box">
                              <Play size={17} />
                            </div>
                            <span>
                              {lang === "ar"
                                ? "برامج بدء التشغيل النشطة"
                                : "Active Startup Programs"}
                            </span>
                          </div>
                          <div className="vibrant-metric">
                            {startupSnapshot.startup_items.length}
                            <span>{lang === "ar" ? "برنامج" : "entries"}</span>
                          </div>
                          <p className="vibrant-sub">
                            {lang === "ar"
                              ? "برامج مسجلة في سجل النظام ومجلد بدء التشغيل"
                              : "Registered across HKCU, HKLM & Startup folder"}
                          </p>
                        </div>
                      </div>

                      <div className="vibrant-card indigo">
                        <div>
                          <div className="vibrant-card-head">
                            <div className="vibrant-icon-box">
                              <Sparkles size={17} />
                            </div>
                            <span>
                              {lang === "ar"
                                ? "ذاكرة DNS والشبكة"
                                : "DNS & Network Cache"}
                            </span>
                          </div>
                          <div className="vibrant-metric">
                            DNS
                            <span>FLUSH</span>
                          </div>
                          <p className="vibrant-sub">
                            {lang === "ar"
                              ? "تفريغ الكاش المؤقت لـ DNS لتسريع فتح المواقع والألعاب"
                              : "Flush stale DNS resolver records instantly"}
                          </p>
                        </div>
                        <button
                          type="button"
                          className="vibrant-pill-btn"
                          disabled={runningBoost}
                          onClick={async () => {
                            setRunningBoost(true);
                            try {
                              const res = await executeSystemBoostAction("flush_dns");
                              setBoostNotice(
                                lang === "ar" ? res.message_ar : res.message_en
                              );
                              const hist = await fetchAuditHistory(50);
                              setAuditEvents(hist);
                            } finally {
                              setRunningBoost(false);
                            }
                          }}
                        >
                          {lang === "ar" ? "تفريغ DNS الآن ⚡" : "Flush DNS Now ⚡"}
                        </button>
                      </div>

                      <div className="vibrant-card coral">
                        <div>
                          <div className="vibrant-card-head">
                            <div className="vibrant-icon-box">
                              <HardDrive size={17} />
                            </div>
                            <span>
                              {lang === "ar"
                                ? "كاش المصغرات والأيقونات"
                                : "Explorer Thumbnail Cache"}
                            </span>
                          </div>
                          <div className="vibrant-metric">
                            {formatBytes(
                              startupSnapshot.thumbcache_reclaimable_bytes
                            )}
                            <span>
                              {startupSnapshot.thumbcache_files_count} files
                            </span>
                          </div>
                          <p className="vibrant-sub">
                            {lang === "ar"
                              ? "تنظيف قواعد بيانات المصغرات غير المقفلة في Explorer"
                              : "Clean unlocked Explorer thumbcache databases"}
                          </p>
                        </div>
                        <button
                          type="button"
                          className="vibrant-pill-btn"
                          disabled={runningBoost}
                          onClick={async () => {
                            setRunningBoost(true);
                            try {
                              const res = await executeSystemBoostAction(
                                "clean_thumbcache"
                              );
                              setBoostNotice(
                                lang === "ar" ? res.message_ar : res.message_en
                              );
                              const [snap, hist] = await Promise.all([
                                fetchStartupAndBoosterStatus(),
                                fetchAuditHistory(50),
                              ]);
                              setStartupSnapshot(snap);
                              setAuditEvents(hist);
                            } finally {
                              setRunningBoost(false);
                            }
                          }}
                        >
                          {lang === "ar"
                            ? "تنظيف الكاش الآن 🧹"
                            : "Clean Cache Now 🧹"}
                        </button>
                      </div>
                    </div>
                  )}

                  {boostNotice && (
                    <div
                      style={{
                        padding: "12px 16px",
                        borderRadius: 14,
                        border: "1px solid rgba(16, 185, 129, 0.5)",
                        background: "rgba(16, 185, 129, 0.14)",
                        fontWeight: 700,
                        fontSize: 13.5,
                        color: "var(--status-ok)",
                      }}
                    >
                      {boostNotice}
                    </div>
                  )}
                </div>

                {startupSnapshot && (
                  <div className="glass-panel">
                    <div className="panel-title">
                      <Play size={18} color="var(--accent-primary)" />
                      <span>
                        {lang === "ar"
                          ? `برامج بدء التشغيل التلقائي عند إقلاع النظام (${startupSnapshot.startup_items.length})`
                          : `Windows Startup Programs (${startupSnapshot.startup_items.length})`}
                      </span>
                    </div>

                    <div className="table-scroll-container">
                      <table className="data-table">
                        <thead>
                          <tr>
                            <th>
                              {lang === "ar" ? "اسم البرنامج" : "Program Name"}
                            </th>
                            <th>
                              {lang === "ar"
                                ? "المسار / أمر التشغيل"
                                : "Command / Path"}
                            </th>
                            <th>{lang === "ar" ? "الموقع" : "Location"}</th>
                            <th>
                              {lang === "ar"
                                ? "تأثير الإقلاع"
                                : "Boot Impact"}
                            </th>
                            <th>{lang === "ar" ? "إجراء" : "Action"}</th>
                          </tr>
                        </thead>
                        <tbody>
                          {startupSnapshot.startup_items.map((item) => (
                            <tr key={item.id}>
                              <td style={{ fontWeight: 800 }}>{item.name}</td>
                              <td
                                className="mono-cell"
                                style={{
                                  fontSize: 11.5,
                                  color: "var(--text-secondary)",
                                  maxWidth: 420,
                                  wordBreak: "break-all",
                                }}
                              >
                                {item.command_or_path}
                              </td>
                              <td>
                                <span className="preset-chip">
                                  {item.location_label}
                                </span>
                              </td>
                              <td>
                                <span
                                  className={`confidence-badge ${
                                    item.impact_estimate === "High"
                                      ? "high"
                                      : item.impact_estimate === "Medium"
                                      ? "medium"
                                      : "low"
                                  }`}
                                >
                                  {item.impact_estimate}
                                </span>
                              </td>
                              <td>
                                {item.can_remove_without_admin ? (
                                  <button
                                    type="button"
                                    className="uninstall-pill-btn"
                                    onClick={async () => {
                                      try {
                                        await removeStartupItem(
                                          item.name,
                                          item.command_or_path,
                                          item.source_type
                                        );
                                        setBoostNotice(
                                          lang === "ar"
                                            ? `تمت إزالة «${item.name}» من بدء التشغيل التلقائي بنجاح!`
                                            : `Removed "${item.name}" from Windows startup!`
                                        );
                                        const [snap, hist] = await Promise.all([
                                          fetchStartupAndBoosterStatus(),
                                          fetchAuditHistory(50),
                                        ]);
                                        setStartupSnapshot(snap);
                                        setAuditEvents(hist);
                                      } catch (err: unknown) {
                                        const payload: UserErrorPayload =
                                          typeof err === "object" &&
                                          err !== null &&
                                          "error_code" in err
                                            ? (err as UserErrorPayload)
                                            : {
                                                error_code:
                                                  "STARTUP_REMOVE_FAILED",
                                                message_en:
                                                  "Could not remove startup entry.",
                                                message_ar:
                                                  "تعذر إزالة العنصر من بدء التشغيل.",
                                                technical_hint: String(err),
                                              };
                                        setError(payload);
                                      }
                                    }}
                                  >
                                    <Trash2 size={12} />
                                    <span>
                                      {lang === "ar"
                                        ? "إيقاف من بدء التشغيل"
                                        : "Disable Startup"}
                                    </span>
                                  </button>
                                ) : (
                                  <span
                                    style={{
                                      fontSize: 11.5,
                                      color: "var(--text-muted)",
                                    }}
                                  >
                                    {lang === "ar"
                                      ? "محمي على مستوى النظام (HKLM)"
                                      : "System-wide (HKLM)"}
                                  </span>
                                )}
                              </td>
                            </tr>
                          ))}
                        </tbody>
                      </table>
                    </div>
                  </div>
                )}
              </>
            )}

            {/* ============================================================
                VIEW 6.6: STANDALONE KERNEL & PROCESS MEMORY ENGINE (SALEEM.MemoryEngine.dll)
               ============================================================ */}
            {activeNav === "memory" && memoryState && (
              <>
                <div className="glass-panel">
                  <div className="panel-header-row">
                    <div>
                      <div className="status-pulse-line">
                        <span className="pulse-dot" />
                        <span>
                          {memoryState.dll_status.dll_file_name} • ABI v
                          {memoryState.dll_status.abi_version} (
                          {memoryState.dll_status.status}) • NT Kernel + Working Sets
                        </span>
                      </div>
                      <h2 style={{ fontSize: 21, fontWeight: 800, marginTop: 4 }}>
                        {lang === "ar"
                          ? "مُفرّغ الرامات الخارق المستقل (SALEEM.MemoryEngine.dll — MemReduct Pro)"
                          : "Standalone Kernel & Process RAM Engine (SALEEM.MemoryEngine.dll)"}
                      </h2>
                      <p
                        style={{
                          fontSize: 13,
                          color: "var(--text-secondary)",
                          marginTop: 4,
                        }}
                      >
                        {lang === "ar"
                          ? "تفريغ عميق لمجموعات العمل (Working Sets)، قوائم الانتظار (Standby List)، الصفحات المعدلة (Modified List)، كاش ملفات النظام، ودمج الصفحات المتطابقة مع مراقبة مستمرة في الخلفية وإشعارات فورية."
                          : "Kernel-level Standby List purge, Working Set eviction, Modified Page List flush, System File Cache reset, and physical page combining with background daemon & Windows notifications."}
                      </p>
                    </div>

                    <div style={{ display: "flex", gap: 10, flexWrap: "wrap" }}>
                      <button
                        type="button"
                        className="icon-pill-btn"
                        onClick={async () => {
                          const st = await fetchMemoryEngineStatus();
                          setMemoryState(st);
                        }}
                      >
                        <RefreshCw size={14} />
                        <span>
                          {lang === "ar" ? "تحديث القراءات" : "Refresh"}
                        </span>
                      </button>

                      <button
                        type="button"
                        className="primary-pill-btn"
                        disabled={cleaningMemory}
                        onClick={() =>
                          void handleExecuteMemoryReduction(
                            memoryState.config.clean_depth,
                            "manual_memory_center"
                          )
                        }
                      >
                        <Cpu size={15} />
                        <span>
                          {cleaningMemory
                            ? lang === "ar"
                              ? "جاري التفريغ العميق للرام..."
                              : "Purging RAM & Standby Cache..."
                            : lang === "ar"
                            ? `تفريغ شامل للرامات الآن (${memoryState.config.clean_depth}) ⚡`
                            : `Purge RAM Now (${memoryState.config.clean_depth}) ⚡`}
                        </span>
                      </button>
                    </div>
                  </div>

                  {/* Live 4-Card Memory Telemetry Bento */}
                  <div className="bento-cards-row">
                    <div className="vibrant-card emerald">
                      <div>
                        <div className="vibrant-card-head">
                          <div className="vibrant-icon-box">
                            <Cpu size={17} />
                          </div>
                          <span>
                            {lang === "ar"
                              ? "الذاكرة الفعلية المستهلكة (Physical RAM)"
                              : "Physical RAM Usage"}
                          </span>
                        </div>
                        <div className="vibrant-metric">
                          {memoryState.snapshot.memory_load_percent}%
                          <span>
                            {formatBytes(memoryState.snapshot.used_physical_bytes)} /{" "}
                            {formatBytes(memoryState.snapshot.total_physical_bytes)}
                          </span>
                        </div>
                        <p className="vibrant-sub">
                          {lang === "ar"
                            ? `الرام الفاضي المتاح فوراً: ${formatBytes(
                                memoryState.snapshot.avail_physical_bytes
                              )}`
                            : `Free Available RAM: ${formatBytes(
                                memoryState.snapshot.avail_physical_bytes
                              )}`}
                        </p>
                      </div>
                    </div>

                    <div className="vibrant-card ocean">
                      <div>
                        <div className="vibrant-card-head">
                          <div className="vibrant-icon-box">
                            <Layers size={17} />
                          </div>
                          <span>
                            {lang === "ar"
                              ? "كاش النظام وقائمة الانتظار (Standby & Cache)"
                              : "System Cache & Standby RAM"}
                          </span>
                        </div>
                        <div className="vibrant-metric">
                          {formatBytes(memoryState.snapshot.system_cache_bytes)}
                          <span>CACHE</span>
                        </div>
                        <p className="vibrant-sub">
                          {lang === "ar"
                            ? `نواة النظام (Kernel): ${formatBytes(
                                memoryState.snapshot.kernel_total_bytes
                              )}`
                            : `Kernel Pool: ${formatBytes(
                                memoryState.snapshot.kernel_total_bytes
                              )}`}
                        </p>
                      </div>
                    </div>

                    <div className="vibrant-card indigo">
                      <div>
                        <div className="vibrant-card-head">
                          <div className="vibrant-icon-box">
                            <HardDrive size={17} />
                          </div>
                          <span>
                            {lang === "ar"
                              ? "الذاكرة الافتراضية (Commit / Pagefile)"
                              : "Commit Charge / Pagefile"}
                          </span>
                        </div>
                        <div className="vibrant-metric">
                          {memoryState.snapshot.commit_load_percent}%
                          <span>
                            {formatBytes(memoryState.snapshot.used_pagefile_bytes)} /{" "}
                            {formatBytes(memoryState.snapshot.total_pagefile_bytes)}
                          </span>
                        </div>
                        <p className="vibrant-sub">
                          {lang === "ar"
                            ? `المتاح في الذاكرة الافتراضية: ${formatBytes(
                                memoryState.snapshot.avail_pagefile_bytes
                              )}`
                            : `Available Commit: ${formatBytes(
                                memoryState.snapshot.avail_pagefile_bytes
                              )}`}
                        </p>
                      </div>
                    </div>

                    <div className="vibrant-card coral">
                      <div>
                        <div className="vibrant-card-head">
                          <div className="vibrant-icon-box">
                            <Activity size={17} />
                          </div>
                          <span>
                            {lang === "ar"
                              ? "العمليات والخيوط النشطة في النظام"
                              : "Active Processes & Threads"}
                          </span>
                        </div>
                        <div className="vibrant-metric">
                          {memoryState.snapshot.process_count}
                          <span>{lang === "ar" ? "عملية نشطة" : "processes"}</span>
                        </div>
                        <p className="vibrant-sub">
                          {lang === "ar"
                            ? `الخيوط (Threads): ${memoryState.snapshot.thread_count.toLocaleString()} • المقابض: ${memoryState.snapshot.handle_count.toLocaleString()}`
                            : `Threads: ${memoryState.snapshot.thread_count.toLocaleString()} • Handles: ${memoryState.snapshot.handle_count.toLocaleString()}`}
                        </p>
                      </div>
                    </div>
                  </div>

                  {/* Last Clean Detailed Report Banner */}
                  {memoryReport && (
                    <div
                      style={{
                        padding: "16px 18px",
                        borderRadius: 16,
                        border: "1px solid rgba(16, 185, 129, 0.55)",
                        background: "rgba(16, 185, 129, 0.12)",
                        display: "flex",
                        flexDirection: "column",
                        gap: 12,
                      }}
                    >
                      <div
                        style={{
                          display: "flex",
                          alignItems: "center",
                          justifyContent: "space-between",
                          flexWrap: "wrap",
                          gap: 10,
                        }}
                      >
                        <div style={{ fontWeight: 800, fontSize: 15 }}>
                          {lang === "ar"
                            ? `⚡ نتيجة آخر تفريغ عبر SALEEM.MemoryEngine.dll (${memoryReport.depth}): تم تحرير ${formatBytes(
                                memoryReport.bytes_freed
                              )} بنجاح!`
                            : `⚡ Last SALEEM.MemoryEngine.dll Purge (${memoryReport.depth}): Freed ${formatBytes(
                                memoryReport.bytes_freed
                              )}!`}
                        </div>
                        <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
                          <span className="badge-ok mono-cell">
                            {memoryReport.before.memory_load_percent}% ➔{" "}
                            {memoryReport.after.memory_load_percent}%
                          </span>
                          <span className="preset-chip mono-cell">
                            {memoryReport.processes_trimmed} /{" "}
                            {memoryReport.processes_scanned}{" "}
                            {lang === "ar" ? "عملية" : "procs"} •{" "}
                            {memoryReport.duration_ms} ms
                          </span>
                        </div>
                      </div>

                      {/* 6-Stage Kernel & Working Set Badges */}
                      <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
                        <span className="preset-chip active">
                          ✓ Working Sets:{" "}
                          {formatBytes(memoryReport.working_set_freed_bytes)}
                        </span>
                        {memoryReport.standby_and_cache_freed_bytes > 0 && (
                          <span className="preset-chip active">
                            ✓ Standby & Cache:{" "}
                            {formatBytes(memoryReport.standby_and_cache_freed_bytes)}
                          </span>
                        )}
                        {memoryReport.areas.system_file_cache_flushed && (
                          <span className="preset-chip active">
                            ✓ System File Cache Flushed
                          </span>
                        )}
                        {memoryReport.areas.standby_list_purged && (
                          <span className="preset-chip active">
                            ✓ Standby List Purged
                          </span>
                        )}
                        {memoryReport.areas.modified_page_list_flushed && (
                          <span className="preset-chip active">
                            ✓ Modified Page List Flushed
                          </span>
                        )}
                        {memoryReport.areas.physical_memory_combined && (
                          <span className="preset-chip active">
                            ✓ Memory Combine ({memoryReport.areas.pages_combined_count}{" "}
                            pages)
                          </span>
                        )}
                        {memoryReport.areas.registry_cache_flushed && (
                          <span className="preset-chip active">
                            ✓ Registry Cache Reconciled
                          </span>
                        )}
                      </div>

                      {/* Top Freed Processes from Last Clean */}
                      {memoryReport.top_freed_processes &&
                        memoryReport.top_freed_processes.length > 0 && (
                          <div
                            style={{
                              display: "grid",
                              gridTemplateColumns:
                                "repeat(auto-fit, minmax(230px, 1fr))",
                              gap: 8,
                            }}
                          >
                            {memoryReport.top_freed_processes
                              .slice(0, 6)
                              .map((proc) => (
                                <div
                                  key={proc.name}
                                  style={{
                                    padding: "8px 12px",
                                    borderRadius: 10,
                                    background: "rgba(0,0,0,0.22)",
                                    border: "1px solid var(--glass-border)",
                                    display: "flex",
                                    justifyContent: "space-between",
                                    alignItems: "center",
                                    fontSize: 12,
                                  }}
                                >
                                  <div>
                                    <div style={{ fontWeight: 800 }}>
                                      {proc.name}{" "}
                                      {proc.instance_count > 1
                                        ? `(${proc.instance_count})`
                                        : ""}
                                    </div>
                                    <div
                                      className="mono-cell"
                                      style={{
                                        fontSize: 11,
                                        color: "var(--text-secondary)",
                                      }}
                                    >
                                      {formatBytes(proc.before_bytes)} ➔{" "}
                                      {formatBytes(proc.after_bytes)}
                                    </div>
                                  </div>
                                  <span
                                    className="mono-cell"
                                    style={{
                                      fontWeight: 800,
                                      color: "var(--status-ok)",
                                    }}
                                  >
                                    -{formatBytes(proc.freed_bytes)}
                                  </span>
                                </div>
                              ))}
                          </div>
                        )}
                    </div>
                  )}

                  {/* Configuration Grid: Depth + Smart Threshold + Interval + Background & Notifications */}
                  <div
                    style={{
                      display: "grid",
                      gridTemplateColumns:
                        "repeat(auto-fit, minmax(260px, 1fr))",
                      gap: 14,
                    }}
                  >
                    {/* 1. Cleaning Depth Mode */}
                    <div
                      style={{
                        padding: "14px 16px",
                        borderRadius: 14,
                        border: "1px solid var(--glass-border)",
                        background: "var(--glass-card-inner)",
                        display: "flex",
                        flexDirection: "column",
                        gap: 8,
                      }}
                    >
                      <div style={{ fontWeight: 800, fontSize: 13.5 }}>
                        {lang === "ar"
                          ? "1. قوة ومستوى تفريغ الرامات:"
                          : "1. RAM Cleaning Depth Mode:"}
                      </div>
                      <p style={{ fontSize: 11.5, color: "var(--text-secondary)" }}>
                        {lang === "ar"
                          ? "الوضع العميق (Deep) يقوم بتفريغ مجموعات العمل + Standby List + كاش الملفات + دمج الصفحات."
                          : "Deep mode purges Working Sets + Kernel Standby List + File Cache + Page Combining."}
                      </p>
                      <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
                        {[
                          {
                            id: "Light",
                            labelAr: "خفيف (Light)",
                            labelEn: "Light",
                          },
                          {
                            id: "Medium",
                            labelAr: "متوسط (Medium)",
                            labelEn: "Medium",
                          },
                          {
                            id: "Deep",
                            labelAr: "عميق وخارق (Deep Kernel)",
                            labelEn: "Deep Kernel",
                          },
                        ].map((mode) => (
                          <button
                            key={mode.id}
                            type="button"
                            className={`preset-chip ${
                              memoryState.config.clean_depth === mode.id
                                ? "active"
                                : ""
                            }`}
                            onClick={() =>
                              void handleUpdateMemoryConfig({
                                clean_depth: mode.id,
                              })
                            }
                          >
                            <span>
                              {lang === "ar" ? mode.labelAr : mode.labelEn}
                            </span>
                          </button>
                        ))}
                      </div>
                    </div>

                    {/* 2. Smart RAM % Threshold Auto-Clean */}
                    <div
                      style={{
                        padding: "14px 16px",
                        borderRadius: 14,
                        border: "1px solid var(--glass-border)",
                        background: "var(--glass-card-inner)",
                        display: "flex",
                        flexDirection: "column",
                        gap: 8,
                      }}
                    >
                      <div style={{ fontWeight: 800, fontSize: 13.5 }}>
                        {lang === "ar"
                          ? "2. التفريغ التلقائي الذكي عند وصول الرام إلى:"
                          : "2. Smart Auto-Purge When RAM Exceeds:"}
                      </div>
                      <p style={{ fontSize: 11.5, color: "var(--text-secondary)" }}>
                        {lang === "ar"
                          ? "يراقب الرام بالخلفية وينظفها فوراً إذا ارتفع الاستهلاك فوق النسبة المحددة."
                          : "Monitors RAM in the background and auto-purges when load exceeds threshold."}
                      </p>
                      <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
                        {[
                          { pct: 0, labelAr: "إيقاف", labelEn: "Off" },
                          { pct: 70, labelAr: "70%", labelEn: "70%" },
                          { pct: 75, labelAr: "75%", labelEn: "75%" },
                          { pct: 80, labelAr: "80%", labelEn: "80%" },
                          { pct: 85, labelAr: "85%", labelEn: "85%" },
                          { pct: 90, labelAr: "90%", labelEn: "90%" },
                        ].map((th) => (
                          <button
                            key={th.pct}
                            type="button"
                            className={`preset-chip ${
                              memoryState.config.auto_threshold_percent ===
                              th.pct
                                ? "active"
                                : ""
                            }`}
                            onClick={() =>
                              void handleUpdateMemoryConfig({
                                auto_threshold_percent: th.pct,
                              })
                            }
                          >
                            <span>
                              {lang === "ar" ? th.labelAr : th.labelEn}
                            </span>
                          </button>
                        ))}
                      </div>
                    </div>

                    {/* 3. Scheduled Periodic Auto-Clean */}
                    <div
                      style={{
                        padding: "14px 16px",
                        borderRadius: 14,
                        border: "1px solid var(--glass-border)",
                        background: "var(--glass-card-inner)",
                        display: "flex",
                        flexDirection: "column",
                        gap: 8,
                      }}
                    >
                      <div style={{ fontWeight: 800, fontSize: 13.5 }}>
                        {lang === "ar"
                          ? "3. جدولة التنظيف التلقائي كل فترة:"
                          : "3. Periodic Auto-Clean Schedule:"}
                      </div>
                      <p style={{ fontSize: 11.5, color: "var(--text-secondary)" }}>
                        {lang === "ar"
                          ? "تفريغ دوري للرامات في الخلفية كل عدد محدد من الدقائق."
                          : "Automatically clean RAM in the background on a fixed timer."}
                      </p>
                      <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
                        {[
                          {
                            mins: 0,
                            labelAr: "يدوي / حسب النسبة",
                            labelEn: "Off",
                          },
                          { mins: 5, labelAr: "5 دقائق", labelEn: "5m" },
                          { mins: 10, labelAr: "10 دقائق", labelEn: "10m" },
                          { mins: 15, labelAr: "15 دقيقة", labelEn: "15m" },
                          { mins: 30, labelAr: "30 دقيقة", labelEn: "30m" },
                          { mins: 60, labelAr: "60 دقيقة", labelEn: "60m" },
                        ].map((iv) => (
                          <button
                            key={iv.mins}
                            type="button"
                            className={`preset-chip ${
                              memoryState.config.auto_interval_minutes ===
                              iv.mins
                                ? "active"
                                : ""
                            }`}
                            onClick={() =>
                              void handleUpdateMemoryConfig({
                                auto_interval_minutes: iv.mins,
                              })
                            }
                          >
                            <span>
                              {lang === "ar" ? iv.labelAr : iv.labelEn}
                            </span>
                          </button>
                        ))}
                      </div>
                    </div>

                    {/* 4. Background Execution, Notifications & Startup Toggles */}
                    <div
                      style={{
                        padding: "14px 16px",
                        borderRadius: 14,
                        border: "1px solid var(--glass-border)",
                        background: "var(--glass-card-inner)",
                        display: "flex",
                        flexDirection: "column",
                        gap: 8,
                      }}
                    >
                      <div style={{ fontWeight: 800, fontSize: 13.5 }}>
                        {lang === "ar"
                          ? "4. العمل في الخلفية والإشعارات وبدء التشغيل:"
                          : "4. Background Daemon, Notifications & Startup:"}
                      </div>
                      <div
                        style={{
                          display: "flex",
                          flexDirection: "column",
                          gap: 7,
                        }}
                      >
                        <label
                          style={{
                            display: "flex",
                            alignItems: "center",
                            gap: 8,
                            fontSize: 12.5,
                            cursor: "pointer",
                          }}
                        >
                          <input
                            type="checkbox"
                            checked={memoryState.config.run_in_background}
                            onChange={(e) =>
                              void handleUpdateMemoryConfig({
                                run_in_background: e.target.checked,
                              })
                            }
                            style={{ accentColor: "var(--accent-primary)" }}
                          />
                          <span>
                            {lang === "ar"
                              ? "إبقاء محرك الرامات يعمل في الخلفية دائماً (عند الإغلاق يصغر للشريط)"
                              : "Keep RAM Engine running in the background on window close"}
                          </span>
                        </label>

                        <label
                          style={{
                            display: "flex",
                            alignItems: "center",
                            gap: 8,
                            fontSize: 12.5,
                            cursor: "pointer",
                          }}
                        >
                          <input
                            type="checkbox"
                            checked={memoryState.config.show_notifications}
                            onChange={(e) =>
                              void handleUpdateMemoryConfig({
                                show_notifications: e.target.checked,
                              })
                            }
                            style={{ accentColor: "var(--accent-primary)" }}
                          />
                          <span>
                            {lang === "ar"
                              ? "إرسال إشعار Windows فوري عند تنظيف الرامات وتوضيح المساحة المحررة"
                              : "Show Windows Toast Notification whenever RAM is cleaned"}
                          </span>
                        </label>

                        <label
                          style={{
                            display: "flex",
                            alignItems: "center",
                            gap: 8,
                            fontSize: 12.5,
                            cursor: "pointer",
                          }}
                        >
                          <input
                            type="checkbox"
                            checked={
                              memoryState.config.launch_app_on_windows_startup
                            }
                            onChange={(e) =>
                              void handleUpdateMemoryConfig({
                                launch_app_on_windows_startup: e.target.checked,
                              })
                            }
                            style={{ accentColor: "var(--accent-primary)" }}
                          />
                          <span>
                            {lang === "ar"
                              ? "تشغيل SALEEM تلقائياً عند إقلاع نظام Windows"
                              : "Launch SALEEM automatically when Windows starts"}
                          </span>
                        </label>

                        <label
                          style={{
                            display: "flex",
                            alignItems: "center",
                            gap: 8,
                            fontSize: 12.5,
                            cursor: "pointer",
                          }}
                        >
                          <input
                            type="checkbox"
                            checked={memoryState.config.clean_on_app_startup}
                            onChange={(e) =>
                              void handleUpdateMemoryConfig({
                                clean_on_app_startup: e.target.checked,
                              })
                            }
                            style={{ accentColor: "var(--accent-primary)" }}
                          />
                          <span>
                            {lang === "ar"
                              ? "تفريغ الرامات تلقائياً أول ما يشتغل التطبيق"
                              : "Clean RAM automatically as soon as SALEEM opens"}
                          </span>
                        </label>
                      </div>
                    </div>
                  </div>
                </div>

                {/* Live Top RAM-Consuming Processes Table with 1-Click Individual Process Trim */}
                <div className="glass-panel">
                  <div className="panel-header-row">
                    <div className="panel-title">
                      <Cpu size={18} color="var(--accent-primary)" />
                      <span>
                        {lang === "ar"
                          ? `أكثر التطبيقات استهلاكاً للرام حالياً (${memoryState.top_processes.length}) — يمكنك تفريغ أي تطبيق بضغطة زر`
                          : `Top RAM-Consuming Processes (${memoryState.top_processes.length}) — 1-Click Per-Process Trim`}
                      </span>
                    </div>
                    {processTrimNotice && (
                      <span
                        className="badge-ok"
                        style={{ fontSize: 12, fontWeight: 700 }}
                      >
                        {processTrimNotice}
                      </span>
                    )}
                  </div>

                  <div className="table-scroll-container">
                    <table className="data-table">
                      <thead>
                        <tr>
                          <th>
                            {lang === "ar" ? "اسم العملية" : "Process Name"}
                          </th>
                          <th>PID</th>
                          <th>
                            {lang === "ar"
                              ? "استهلاك الرام الفعلي (Working Set)"
                              : "Working Set RAM"}
                          </th>
                          <th>
                            {lang === "ar"
                              ? "الذاكرة الخاصة (Private)"
                              : "Private Bytes"}
                          </th>
                          <th>
                            {lang === "ar"
                              ? "أقصى استهلاك (Peak)"
                              : "Peak Working Set"}
                          </th>
                          <th>{lang === "ar" ? "تفريغ فوري" : "Action"}</th>
                        </tr>
                      </thead>
                      <tbody>
                        {memoryState.top_processes.map((proc) => (
                          <tr key={proc.pid}>
                            <td style={{ fontWeight: 800 }}>{proc.name}</td>
                            <td className="mono-cell">{proc.pid}</td>
                            <td
                              className="mono-cell"
                              style={{
                                fontWeight: 800,
                                color: "var(--accent-primary)",
                              }}
                            >
                              {formatBytes(proc.working_set_bytes)}
                            </td>
                            <td className="mono-cell">
                              {formatBytes(proc.private_bytes)}
                            </td>
                            <td
                              className="mono-cell"
                              style={{ color: "var(--text-secondary)" }}
                            >
                              {formatBytes(proc.peak_working_set_bytes)}
                            </td>
                            <td>
                              <button
                                type="button"
                                className="primary-pill-btn"
                                style={{
                                  padding: "5px 12px",
                                  fontSize: 11.5,
                                }}
                                disabled={trimmingPid === proc.pid}
                                onClick={async () => {
                                  setTrimmingPid(proc.pid);
                                  try {
                                    const freed = await trimSingleProcess(
                                      proc.pid,
                                      proc.name
                                    );
                                    setProcessTrimNotice(
                                      lang === "ar"
                                        ? `تم تفريغ ${formatBytes(
                                            freed
                                          )} من ${proc.name} (#${proc.pid})!`
                                        : `Freed ${formatBytes(freed)} from ${
                                            proc.name
                                          } (#${proc.pid})!`
                                    );
                                    const st = await fetchMemoryEngineStatus();
                                    setMemoryState(st);
                                  } finally {
                                    setTrimmingPid(null);
                                  }
                                }}
                              >
                                <Cpu size={12} />
                                <span>
                                  {trimmingPid === proc.pid
                                    ? lang === "ar"
                                      ? "جاري..."
                                      : "Trimming..."
                                    : lang === "ar"
                                    ? "تفريغ التطبيق ⚡"
                                    : "Trim RAM ⚡"}
                                </span>
                              </button>
                            </td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                </div>
              </>
            )}

            {/* ============================================================
                VIEW 7: AUDIT HISTORY
               ============================================================ */}
            {activeNav === "history" && (
              <div className="glass-panel">
                <div>
                  <h2 style={{ fontSize: 20, fontWeight: 800 }}>
                    {t.history.title}
                  </h2>
                  <p style={{ fontSize: 13, color: "var(--text-secondary)" }}>
                    {t.history.subtitle}
                  </p>
                </div>

                {auditEvents.length === 0 ? (
                  <p>{t.history.empty}</p>
                ) : (
                  <table className="data-table">
                    <thead>
                      <tr>
                        <th>{t.history.colTime}</th>
                        <th>{t.history.colType}</th>
                        <th>{t.history.colSubject}</th>
                        <th>{t.history.colResult}</th>
                        <th>{t.history.colDetails}</th>
                      </tr>
                    </thead>
                    <tbody>
                      {auditEvents.map((ev) => (
                        <tr key={ev.id}>
                          <td className="mono-cell">{ev.timestamp}</td>
                          <td className="mono-cell">{ev.event_type}</td>
                          <td className="mono-cell">{ev.subject}</td>
                          <td>
                            <span className="badge-ok">{ev.result}</span>
                          </td>
                          <td>{ev.details_redacted}</td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                )}
              </div>
            )}

            {/* ============================================================
                VIEW 5: INDEPENDENT SETTINGS, UPDATES CENTER & NATIVE DLLs
               ============================================================ */}
            {activeNav === "settings" && status && (
              <>
                {/* Independent Sub-Navigation Bar for Settings */}
                <div className="glass-panel" style={{ padding: "14px 18px" }}>
                  <div
                    style={{
                      display: "flex",
                      alignItems: "center",
                      justifyContent: "space-between",
                      flexWrap: "wrap",
                      gap: 12,
                    }}
                  >
                    <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
                      {[
                        {
                          id: "updates" as const,
                          labelAr: "🔗 مركز التحديثات المستقل (Updates Center)",
                          labelEn: "🔗 Independent Updates Center",
                        },
                        {
                          id: "engines" as const,
                          labelAr: "⚙️ محركات الـ DLL المستقلة (Native DLLs)",
                          labelEn: "⚙️ Standalone Native DLL Engines",
                        },
                        {
                          id: "policy" as const,
                          labelAr: "🛡️ الأمان وقاعدة البيانات (Safety & DB)",
                          labelEn: "🛡️ Safety Policy & SQLite DB",
                        },
                      ].map((tab) => (
                        <button
                          key={tab.id}
                          type="button"
                          className={`preset-chip ${
                            settingsTab === tab.id ? "active" : ""
                          }`}
                          onClick={() => setSettingsTab(tab.id)}
                        >
                          <span>
                            {lang === "ar" ? tab.labelAr : tab.labelEn}
                          </span>
                        </button>
                      ))}
                    </div>

                    <span className="badge-ok mono-cell">
                      v{cloudConfig?.current_version || status.app_version}
                    </span>
                  </div>
                </div>

                {settingsTab === "updates" && (
                  <div className="glass-panel">
                    <div className="panel-header-row">
                      <div className="panel-title">
                        <RefreshCw size={18} color="var(--accent-primary)" />
                        <span>
                          {lang === "ar"
                            ? "مركز التحديثات وسجل الإصدارات (Manual Review Before Install)"
                            : "Official Updates & Release Review Center"}
                        </span>
                      </div>

                      <div
                        style={{ display: "flex", gap: 8, alignItems: "center" }}
                      >
                        <span className="badge-ok">
                          {lang === "ar"
                            ? `الإصدار المثبت حالياً: v${
                                cloudConfig?.current_version ||
                                status.app_version
                              }`
                            : `Installed Version: v${
                                cloudConfig?.current_version ||
                                status.app_version
                              }`}
                        </span>
                        <button
                          type="button"
                          className="primary-pill-btn"
                          onClick={() => void handleCheckCloudUpdate()}
                          disabled={checkingUpdate}
                        >
                          <RefreshCw size={13} />
                          <span>
                            {checkingUpdate
                              ? lang === "ar"
                                ? "جاري فحص التحديثات..."
                                : "Checking for updates..."
                              : lang === "ar"
                              ? "فحص التحديثات المتوفرة"
                              : "Check for Updates"}
                          </span>
                        </button>
                      </div>
                    </div>

                    <p style={{ fontSize: 13, color: "var(--text-secondary)" }}>
                      {lang === "ar"
                        ? "لا يقوم SALEEM بتثبيت أي تحديث تلقائياً دون علمك؛ يمكنك فحص التحديث ومراجعة نوعه وتاريخه وقائمة المميزات الجديدة أولاً قبل الضغط على زر التثبيت."
                        : "SALEEM never installs updates automatically without your consent. Check for updates to review the release type, publication date, and full changelog before choosing to install."}
                    </p>

                    {updateCheckResult && (
                      <div
                        style={{
                          padding: "18px 20px",
                          borderRadius: 18,
                          border: updateCheckResult.update_available
                            ? "1px solid rgba(16, 185, 129, 0.55)"
                            : "1px solid var(--glass-border)",
                          background: updateCheckResult.update_available
                            ? "rgba(16, 185, 129, 0.12)"
                            : "var(--glass-card-inner)",
                          display: "flex",
                          flexDirection: "column",
                          gap: 14,
                        }}
                      >
                        {updateCheckResult.update_available &&
                        updateCheckResult.latest_release ? (
                          <>
                            <div
                              style={{
                                display: "flex",
                                alignItems: "center",
                                justifyContent: "space-between",
                                flexWrap: "wrap",
                                gap: 12,
                              }}
                            >
                              <div
                                style={{
                                  display: "flex",
                                  flexDirection: "column",
                                  gap: 6,
                                }}
                              >
                                <div
                                  style={{
                                    fontWeight: 800,
                                    fontSize: 16,
                                    color: "var(--status-ok)",
                                  }}
                                >
                                  {lang === "ar"
                                    ? `🎉 يتوفر تحديث جديد: الإصدار v${updateCheckResult.latest_release.version}`
                                    : `🎉 New Update Available: v${updateCheckResult.latest_release.version}`}
                                </div>

                                {/* Update Type, Date, and Size Badges */}
                                <div
                                  style={{
                                    display: "flex",
                                    gap: 8,
                                    flexWrap: "wrap",
                                    alignItems: "center",
                                  }}
                                >
                                  <span
                                    className={
                                      updateCheckResult.latest_release
                                        .is_mandatory
                                        ? "badge-warn"
                                        : "badge-ok"
                                    }
                                  >
                                    {lang === "ar"
                                      ? `نوع التحديث: ${
                                          updateCheckResult.latest_release
                                            .is_mandatory
                                            ? "تحديث أمني / هام (Mandatory)"
                                            : "تحديث ميزات مستقر (Stable Feature Release)"
                                        } [${
                                          updateCheckResult.latest_release
                                            .channel
                                        }]`
                                      : `Type: ${
                                          updateCheckResult.latest_release
                                            .is_mandatory
                                            ? "Mandatory Security Release"
                                            : "Stable Feature Update"
                                        } [${
                                          updateCheckResult.latest_release
                                            .channel
                                        }]`}
                                  </span>

                                  <span className="preset-chip mono-cell">
                                    {lang === "ar"
                                      ? `تاريخ الإصدار: ${new Date(
                                          updateCheckResult.latest_release.published_at
                                        ).toLocaleString("en-GB")}`
                                      : `Published: ${new Date(
                                          updateCheckResult.latest_release.published_at
                                        ).toLocaleString("en-GB")}`}
                                  </span>

                                  <span className="preset-chip mono-cell">
                                    {formatBytes(
                                      updateCheckResult.latest_release
                                        .file_size_bytes
                                    )}
                                  </span>
                                </div>
                              </div>

                              <button
                                type="button"
                                className="primary-pill-btn"
                                disabled={downloadingUpdate}
                                onClick={() =>
                                  void handleDownloadAndInstallUpdate(
                                    updateCheckResult.latest_release!.version,
                                    updateCheckResult.latest_release!
                                      .download_url,
                                    updateCheckResult.latest_release!
                                      .sha256_checksum
                                  )
                                }
                              >
                                <RefreshCw size={14} />
                                <span>
                                  {downloadingUpdate
                                    ? lang === "ar"
                                      ? "جاري التحميل والتثبيت في الخلفية..."
                                      : "Downloading & Installing..."
                                    : updateInstallDone
                                    ? lang === "ar"
                                      ? "تم التحميل وبدء التثبيت ✓"
                                      : "Update Installed ✓"
                                    : lang === "ar"
                                    ? "موافق — تحميل وتثبيت التحديث الآن"
                                    : "Approve — Download & Install Update Now"}
                                </span>
                              </button>
                            </div>

                            {/* Detailed What's New / Features Box Before Updating */}
                            <div
                              style={{
                                padding: "14px 16px",
                                borderRadius: 12,
                                border: "1px solid var(--glass-border)",
                                background: "rgba(15, 23, 42, 0.35)",
                                display: "flex",
                                flexDirection: "column",
                                gap: 6,
                              }}
                            >
                              <div
                                style={{
                                  fontWeight: 800,
                                  fontSize: 13,
                                  color: "var(--text-primary)",
                                }}
                              >
                                {lang === "ar"
                                  ? "📋 مميزات وتفاصيل هذا التحديث (راجعها قبل التثبيت):"
                                  : "📋 What's New in This Update (Review before installing):"}
                              </div>
                              <div
                                style={{
                                  fontSize: 13.5,
                                  color: "var(--text-secondary)",
                                  lineHeight: 1.65,
                                  whiteSpace: "pre-line",
                                }}
                              >
                                {lang === "ar"
                                  ? updateCheckResult.latest_release
                                      .release_notes_ar
                                  : updateCheckResult.latest_release
                                      .release_notes_en}
                              </div>
                            </div>

                            <div
                              className="mono-cell"
                              style={{
                                fontSize: 11,
                                color: "var(--text-muted)",
                              }}
                            >
                              SHA-256:{" "}
                              {
                                updateCheckResult.latest_release
                                  .sha256_checksum
                              }
                            </div>
                          </>
                        ) : (
                          <div style={{ fontWeight: 700, fontSize: 13.5 }}>
                            {lang === "ar"
                              ? `أنت تستخدم أحدث إصدار رسمي بالفعل (v${updateCheckResult.current_version}).`
                              : `You are running the latest official release (v${updateCheckResult.current_version}).`}
                          </div>
                        )}
                      </div>
                    )}
                  </div>
                )}

                {settingsTab === "engines" && (
                  <div className="glass-panel">
                    <div className="panel-title">
                      <Cpu size={18} color="var(--accent-primary)" />
                      <span>{t.overview.enginesTitle}</span>
                    </div>

                    <table className="data-table">
                      <thead>
                        <tr>
                          <th>Engine</th>
                          <th>Target DLL</th>
                          <th>{t.overview.loadMode}</th>
                          <th>{t.overview.abiVersion}</th>
                          <th>Status</th>
                        </tr>
                      </thead>
                      <tbody>
                        {status.engines.map((eng) => (
                          <tr key={eng.logical_name}>
                            <td style={{ fontWeight: 700 }}>
                              {eng.logical_name}
                            </td>
                            <td className="mono-cell">{eng.dll_file_name}</td>
                            <td className="mono-cell">{eng.loaded_via}</td>
                            <td className="mono-cell">
                              v{eng.abi_version} (expected v
                              {eng.expected_abi_version})
                            </td>
                            <td>
                              <span
                                className={
                                  eng.is_compatible ? "badge-ok" : "badge-warn"
                                }
                              >
                                {eng.status}
                              </span>
                            </td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                )}

                {settingsTab === "policy" && (
                  <>
                    <div className="glass-panel">
                      <div className="panel-title">
                        <ShieldCheck size={18} color="var(--accent-primary)" />
                        <span>{t.settings.safetyPolicyTitle}</span>
                      </div>
                      <p style={{ color: "var(--text-secondary)" }}>
                        {t.settings.safetyPolicyDesc}
                      </p>
                    </div>

                    <div className="glass-panel">
                      <div className="panel-title">
                        <Database size={18} color="var(--accent-primary)" />
                        <span>{t.overview.databaseTitle}</span>
                      </div>
                      <div className="mono-cell">
                        Schema v{status.database.schema_version} •{" "}
                        {status.database.tables_count} Tables •{" "}
                        {status.database.scans_count} Scans Recorded •{" "}
                        {status.database.audit_events_count} Audit Events
                      </div>
                    </div>
                  </>
                )}
              </>
            )}

            {/* ============================================================
                SCHEDULED FUTURE PHASES (NO FAKE DATA)
               ============================================================ */}
            {activeNav !== "overview" &&
              activeNav !== "storage" &&
              activeNav !== "large_files" &&
              activeNav !== "duplicates" &&
              activeNav !== "cleanup" &&
              activeNav !== "applications" &&
              activeNav !== "startup" &&
              activeNav !== "history" &&
              activeNav !== "settings" && (
                <div className="glass-panel">
                  <div className="panel-title">
                    <FileSearch size={18} color="var(--accent-primary)" />
                    <span>
                      {t.nav[activeNav]} — {t.phasePlaceholder.scheduledTitle}
                    </span>
                  </div>
                  <p style={{ color: "var(--text-secondary)" }}>
                    {t.phasePlaceholder.scheduledDesc}
                  </p>
                </div>
              )}
          </div>
        </main>
      </div>
    </div>
  );
}
