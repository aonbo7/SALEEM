use chrono::Utc;
use saleem_core::{SaleemResult, SALEEM_ABI_VERSION};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

#[no_mangle]
pub extern "C" fn memory_engine_abi_version() -> u32 {
    SALEEM_ABI_VERSION
}

/// C-ABI entry point so external tools can invoke SALEEM.MemoryEngine.dll directly.
/// Returns freed bytes count (`u64`). `depth_mode`: 0 = Light, 1 = Medium, 2 = Deep.
#[no_mangle]
pub extern "C" fn memory_engine_reduce_ram(depth_mode: u32) -> u64 {
    let depth = match depth_mode {
        0 => MemoryCleanDepth::Light,
        1 => MemoryCleanDepth::Medium,
        _ => MemoryCleanDepth::Deep,
    };
    match execute_memory_reduction(depth) {
        Ok(rep) => rep.bytes_freed,
        Err(_) => 0,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryCleanDepth {
    Light,
    Medium,
    Deep,
}

impl MemoryCleanDepth {
    pub fn from_str_mode(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "light" => Self::Light,
            "medium" => Self::Medium,
            _ => Self::Deep,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Light => "Light",
            Self::Medium => "Medium",
            Self::Deep => "Deep",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySnapshot {
    pub memory_load_percent: u32,
    pub total_physical_bytes: u64,
    pub avail_physical_bytes: u64,
    pub used_physical_bytes: u64,
    pub total_pagefile_bytes: u64,
    pub avail_pagefile_bytes: u64,
    pub used_pagefile_bytes: u64,
    pub commit_load_percent: u32,
    pub system_cache_bytes: u64,
    pub kernel_total_bytes: u64,
    pub kernel_paged_bytes: u64,
    pub kernel_nonpaged_bytes: u64,
    pub process_count: u32,
    pub thread_count: u32,
    pub handle_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessMemoryEntry {
    pub pid: u32,
    pub name: String,
    pub working_set_bytes: u64,
    pub private_bytes: u64,
    pub peak_working_set_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessTrimDiff {
    pub name: String,
    pub instance_count: usize,
    pub before_bytes: u64,
    pub after_bytes: u64,
    pub freed_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryCleanAreas {
    pub working_sets_cleaned: bool,
    pub system_file_cache_flushed: bool,
    pub modified_page_list_flushed: bool,
    pub standby_list_purged: bool,
    pub low_priority_standby_purged: bool,
    pub physical_memory_combined: bool,
    pub registry_cache_flushed: bool,
    pub pages_combined_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryCleanReport {
    pub depth: String,
    pub before: MemorySnapshot,
    pub after: MemorySnapshot,
    pub bytes_freed: u64,
    pub working_set_freed_bytes: u64,
    pub standby_and_cache_freed_bytes: u64,
    pub processes_scanned: usize,
    pub processes_trimmed: usize,
    pub areas: MemoryCleanAreas,
    pub top_freed_processes: Vec<ProcessTrimDiff>,
    pub duration_ms: u64,
    pub completed_at: String,
    pub trigger_source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEngineConfig {
    pub clean_depth: String,                // "Light" | "Medium" | "Deep"
    pub auto_interval_minutes: u32,         // 0 = Off, 5, 10, 15, 30, 60
    pub auto_threshold_percent: u32,        // 0 = Off, 70, 75, 80, 85, 90
    pub clean_on_app_startup: bool,
    pub launch_app_on_windows_startup: bool,
    pub run_in_background: bool,
    pub show_notifications: bool,
}

#[cfg(windows)]
#[link(name = "ntdll")]
extern "system" {
    fn NtSetSystemInformation(
        system_information_class: u32,
        system_information: *mut std::ffi::c_void,
        system_information_length: u32,
    ) -> i32;
}

#[cfg(windows)]
const SYSTEM_MEMORY_LIST_INFORMATION: u32 = 80; // 0x50
#[cfg(windows)]
const SYSTEM_FILECACHE_INFORMATION_EX: u32 = 81; // 0x51
#[cfg(windows)]
const SYSTEM_COMBINE_PHYSICAL_MEMORY_INFORMATION: u32 = 130; // 0x82
#[cfg(windows)]
const SYSTEM_REGISTRY_RECONCILIATION_INFORMATION: u32 = 155; // 0x9B

#[cfg(windows)]
const MEMORY_EMPTY_WORKING_SETS: i32 = 2;
#[cfg(windows)]
const MEMORY_FLUSH_MODIFIED_LIST: i32 = 3;
#[cfg(windows)]
const MEMORY_PURGE_STANDBY_LIST: i32 = 4;
#[cfg(windows)]
const MEMORY_PURGE_LOW_PRIORITY_STANDBY_LIST: i32 = 5;

#[cfg(windows)]
#[repr(C)]
struct MemoryCombineInformationEx {
    handle: isize,
    pages_combined: usize,
    flags: u32,
}

#[cfg(windows)]
#[repr(C)]
struct SystemFileCacheInformation {
    current_size: usize,
    peak_size: usize,
    page_fault_count: u32,
    minimum_working_set: usize,
    maximum_working_set: usize,
    current_size_including_transition_in_pages: usize,
    peak_size_including_transition_in_pages: usize,
    transition_repurpose_count: u32,
    flags: u32,
}

/// Enables `SeProfileSingleProcessPrivilege`, `SeIncreaseQuotaPrivilege`, and `SeDebugPrivilege`
/// on the current process token so both user-mode and kernel-mode memory reduction APIs work at maximum power.
#[cfg(windows)]
unsafe fn enable_memory_privileges() {
    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, LUID};
    use windows_sys::Win32::Security::{
        AdjustTokenPrivileges, LookupPrivilegeValueW, LUID_AND_ATTRIBUTES, SE_PRIVILEGE_ENABLED,
        TOKEN_ADJUST_PRIVILEGES, TOKEN_PRIVILEGES, TOKEN_QUERY,
    };
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    let mut token: HANDLE = std::ptr::null_mut();
    if OpenProcessToken(
        GetCurrentProcess(),
        TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
        &mut token,
    ) == 0
        || token.is_null()
    {
        return;
    }

    let priv_names = [
        "SeProfileSingleProcessPrivilege\0",
        "SeIncreaseQuotaPrivilege\0",
        "SeDebugPrivilege\0",
    ];

    for name in priv_names {
        let wide: Vec<u16> = name.encode_utf16().collect();
        let mut luid: LUID = std::mem::zeroed();
        if LookupPrivilegeValueW(std::ptr::null(), wide.as_ptr(), &mut luid) != 0 {
            let mut tp = TOKEN_PRIVILEGES {
                PrivilegeCount: 1,
                Privileges: [LUID_AND_ATTRIBUTES {
                    Luid: luid,
                    Attributes: SE_PRIVILEGE_ENABLED,
                }],
            };
            let _ = AdjustTokenPrivileges(
                token,
                0,
                &mut tp,
                std::mem::size_of::<TOKEN_PRIVILEGES>() as u32,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
        }
    }

    CloseHandle(token);
}

pub fn get_memory_snapshot() -> MemorySnapshot {
    #[cfg(windows)]
    unsafe {
        use windows_sys::Win32::System::ProcessStatus::{
            K32GetPerformanceInfo, PERFORMANCE_INFORMATION,
        };
        use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};

        let mut mem: MEMORYSTATUSEX = std::mem::zeroed();
        mem.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;

        let mut perf: PERFORMANCE_INFORMATION = std::mem::zeroed();
        perf.cb = std::mem::size_of::<PERFORMANCE_INFORMATION>() as u32;
        let has_perf = K32GetPerformanceInfo(&mut perf, perf.cb) != 0;

        if GlobalMemoryStatusEx(&mut mem) != 0 {
            let used_phys = mem.ullTotalPhys.saturating_sub(mem.ullAvailPhys);
            let used_page = mem.ullTotalPageFile.saturating_sub(mem.ullAvailPageFile);
            let commit_pct = if mem.ullTotalPageFile > 0 {
                ((used_page as f64 / mem.ullTotalPageFile as f64) * 100.0).round() as u32
            } else {
                0
            };
            let page_size = if has_perf { perf.PageSize as u64 } else { 4096 };

            return MemorySnapshot {
                memory_load_percent: mem.dwMemoryLoad,
                total_physical_bytes: mem.ullTotalPhys,
                avail_physical_bytes: mem.ullAvailPhys,
                used_physical_bytes: used_phys,
                total_pagefile_bytes: mem.ullTotalPageFile,
                avail_pagefile_bytes: mem.ullAvailPageFile,
                used_pagefile_bytes: used_page,
                commit_load_percent: commit_pct.min(100),
                system_cache_bytes: if has_perf {
                    (perf.SystemCache as u64) * page_size
                } else {
                    0
                },
                kernel_total_bytes: if has_perf {
                    (perf.KernelTotal as u64) * page_size
                } else {
                    0
                },
                kernel_paged_bytes: if has_perf {
                    (perf.KernelPaged as u64) * page_size
                } else {
                    0
                },
                kernel_nonpaged_bytes: if has_perf {
                    (perf.KernelNonpaged as u64) * page_size
                } else {
                    0
                },
                process_count: if has_perf { perf.ProcessCount } else { 0 },
                thread_count: if has_perf { perf.ThreadCount } else { 0 },
                handle_count: if has_perf { perf.HandleCount } else { 0 },
            };
        }
    }

    MemorySnapshot {
        memory_load_percent: 0,
        total_physical_bytes: 0,
        avail_physical_bytes: 0,
        used_physical_bytes: 0,
        total_pagefile_bytes: 0,
        avail_pagefile_bytes: 0,
        used_pagefile_bytes: 0,
        commit_load_percent: 0,
        system_cache_bytes: 0,
        kernel_total_bytes: 0,
        kernel_paged_bytes: 0,
        kernel_nonpaged_bytes: 0,
        process_count: 0,
        thread_count: 0,
        handle_count: 0,
    }
}

/// Returns top RAM-consuming processes sorted by Working Set size descending.
pub fn get_top_memory_processes(limit: usize) -> Vec<ProcessMemoryEntry> {
    let mut list = Vec::new();

    #[cfg(windows)]
    unsafe {
        use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
        use windows_sys::Win32::System::Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
            TH32CS_SNAPPROCESS,
        };
        use windows_sys::Win32::System::ProcessStatus::{
            K32GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS, PROCESS_MEMORY_COUNTERS_EX,
        };
        use windows_sys::Win32::System::Threading::{
            OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_QUERY_LIMITED_INFORMATION,
        };

        enable_memory_privileges();

        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snap != INVALID_HANDLE_VALUE && !snap.is_null() {
            let mut entry: PROCESSENTRY32W = std::mem::zeroed();
            entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

            if Process32FirstW(snap, &mut entry) != 0 {
                loop {
                    let pid = entry.th32ProcessID;
                    if pid > 4 {
                        let name = String::from_utf16_lossy(&entry.szExeFile)
                            .trim_end_matches('\0')
                            .to_string();

                        let mut handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
                        if handle.is_null() {
                            handle = OpenProcess(PROCESS_QUERY_INFORMATION, 0, pid);
                        }
                        if !handle.is_null() {
                            let mut pmc: PROCESS_MEMORY_COUNTERS_EX = std::mem::zeroed();
                            pmc.cb = std::mem::size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32;
                            if K32GetProcessMemoryInfo(
                                handle,
                                &mut pmc as *mut _ as *mut PROCESS_MEMORY_COUNTERS,
                                pmc.cb,
                            ) != 0
                            {
                                list.push(ProcessMemoryEntry {
                                    pid,
                                    name,
                                    working_set_bytes: pmc.WorkingSetSize as u64,
                                    private_bytes: pmc.PrivateUsage as u64,
                                    peak_working_set_bytes: pmc.PeakWorkingSetSize as u64,
                                });
                            }
                            CloseHandle(handle);
                        }
                    }

                    if Process32NextW(snap, &mut entry) == 0 {
                        break;
                    }
                }
            }
            CloseHandle(snap);
        }
    }

    list.sort_by(|a, b| b.working_set_bytes.cmp(&a.working_set_bytes));
    list.truncate(limit);
    list
}

/// Trims a single process by PID using aggressive working set eviction and returns freed bytes.
pub fn trim_single_process(pid: u32) -> SaleemResult<u64> {
    #[cfg(windows)]
    unsafe {
        use windows_sys::Win32::Foundation::CloseHandle;
        use windows_sys::Win32::System::Memory::{
            SetProcessWorkingSetSizeEx, QUOTA_LIMITS_HARDWS_MAX_ENABLE,
            QUOTA_LIMITS_HARDWS_MIN_DISABLE,
        };
        use windows_sys::Win32::System::ProcessStatus::{
            K32EmptyWorkingSet, K32GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS,
            PROCESS_MEMORY_COUNTERS_EX,
        };
        use windows_sys::Win32::System::Threading::{
            OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_QUERY_LIMITED_INFORMATION,
            PROCESS_SET_QUOTA,
        };

        enable_memory_privileges();

        let mut handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SET_QUOTA, 0, pid);
        if handle.is_null() {
            handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_SET_QUOTA, 0, pid);
        }
        if handle.is_null() {
            return Ok(0);
        }

        let mut before_ws = 0u64;
        let mut pmc: PROCESS_MEMORY_COUNTERS_EX = std::mem::zeroed();
        pmc.cb = std::mem::size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32;
        if K32GetProcessMemoryInfo(
            handle,
            &mut pmc as *mut _ as *mut PROCESS_MEMORY_COUNTERS,
            pmc.cb,
        ) != 0
        {
            before_ws = pmc.WorkingSetSize as u64;
        }

        let _ = SetProcessWorkingSetSizeEx(
            handle,
            usize::MAX,
            usize::MAX,
            QUOTA_LIMITS_HARDWS_MIN_DISABLE | QUOTA_LIMITS_HARDWS_MAX_ENABLE,
        );
        let _ = K32EmptyWorkingSet(handle);

        let mut after_ws = before_ws;
        if K32GetProcessMemoryInfo(
            handle,
            &mut pmc as *mut _ as *mut PROCESS_MEMORY_COUNTERS,
            pmc.cb,
        ) != 0
        {
            after_ws = pmc.WorkingSetSize as u64;
        }

        CloseHandle(handle);
        return Ok(before_ws.saturating_sub(after_ws));
    }

    #[cfg(not(windows))]
    Ok(0)
}

/// Executes full MemReduct-grade Kernel + User-Mode RAM reduction:
/// - Enables `SeProfileSingleProcessPrivilege`, `SeIncreaseQuotaPrivilege`, `SeDebugPrivilege`
/// - Multi-pass `SetProcessWorkingSetSizeEx(..., QUOTA_LIMITS_HARDWS_MIN_DISABLE | QUOTA_LIMITS_HARDWS_MAX_ENABLE)` + `K32EmptyWorkingSet`
/// - Kernel `NtSetSystemInformation`:
///   - `MemoryEmptyWorkingSets` (Global kernel working set purge)
///   - `SetSystemFileCacheSize` + `SystemFileCacheInformationEx` (System File Cache purge)
///   - `SystemRegistryReconciliationInformation` (Registry cache flush)
///   - `SystemCombinePhysicalMemoryInformation` (Physical page deduplication)
///   - `MemoryFlushModifiedList` (Modified page list flush)
///   - `MemoryPurgeLowPriorityStandbyList` & `MemoryPurgeStandbyList` (Standby RAM reclaim)
pub fn execute_memory_reduction(depth: MemoryCleanDepth) -> SaleemResult<MemoryCleanReport> {
    execute_memory_reduction_with_source(depth, "manual")
}

pub fn execute_memory_reduction_with_source(
    depth: MemoryCleanDepth,
    trigger_source: &str,
) -> SaleemResult<MemoryCleanReport> {
    let start = Instant::now();
    let before = get_memory_snapshot();
    let mut processes_scanned = 0usize;
    let mut processes_trimmed = 0usize;
    let mut total_ws_before: HashMap<String, (usize, u64)> = HashMap::new();
    let mut total_ws_after: HashMap<String, u64> = HashMap::new();

    let mut areas = MemoryCleanAreas {
        working_sets_cleaned: false,
        system_file_cache_flushed: false,
        modified_page_list_flushed: false,
        standby_list_purged: false,
        low_priority_standby_purged: false,
        physical_memory_combined: false,
        registry_cache_flushed: false,
        pages_combined_count: 0,
    };

    #[cfg(windows)]
    unsafe {
        use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
        use windows_sys::Win32::System::Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
            TH32CS_SNAPPROCESS,
        };
        use windows_sys::Win32::System::Memory::{
            GetProcessHeap, HeapCompact, SetProcessWorkingSetSizeEx, SetSystemFileCacheSize,
            QUOTA_LIMITS_HARDWS_MAX_ENABLE, QUOTA_LIMITS_HARDWS_MIN_DISABLE,
        };
        use windows_sys::Win32::System::ProcessStatus::{
            K32EmptyWorkingSet, K32GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS,
            PROCESS_MEMORY_COUNTERS_EX,
        };
        use windows_sys::Win32::System::Threading::{
            GetCurrentProcessId, OpenProcess, PROCESS_QUERY_INFORMATION,
            PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SET_QUOTA,
        };

        // 1. Enable privileges required for both process-level and kernel-level RAM reclamation
        enable_memory_privileges();

        // Compact our own heap first
        let heap = GetProcessHeap();
        if !heap.is_null() {
            let _ = HeapCompact(heap, 0);
        }

        let current_pid = GetCurrentProcessId();
        let passes = match depth {
            MemoryCleanDepth::Light => 1,
            MemoryCleanDepth::Medium => 2,
            MemoryCleanDepth::Deep => 3,
        };

        // 2. Pass 1..N: User-mode process working set eviction (with hard minimum floor disable)
        for pass in 0..passes {
            let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
            if snap != INVALID_HANDLE_VALUE && !snap.is_null() {
                let mut entry: PROCESSENTRY32W = std::mem::zeroed();
                entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

                if Process32FirstW(snap, &mut entry) != 0 {
                    loop {
                        let pid = entry.th32ProcessID;
                        if pass == 0 {
                            processes_scanned += 1;
                        }

                        if pid > 4 {
                            let raw_name = String::from_utf16_lossy(&entry.szExeFile)
                                .trim_end_matches('\0')
                                .to_string();
                            let exe_lower = raw_name.to_ascii_lowercase();

                            // In Light mode, skip active audio/compositor processes
                            let skip_in_light = depth == MemoryCleanDepth::Light
                                && (pid == current_pid
                                    || exe_lower == "audiodg.exe"
                                    || exe_lower == "dwm.exe");

                            if !skip_in_light {
                                let mut handle = OpenProcess(
                                    PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SET_QUOTA,
                                    0,
                                    pid,
                                );
                                if handle.is_null() {
                                    handle = OpenProcess(
                                        PROCESS_QUERY_INFORMATION | PROCESS_SET_QUOTA,
                                        0,
                                        pid,
                                    );
                                }
                                if handle.is_null() {
                                    handle = OpenProcess(PROCESS_SET_QUOTA, 0, pid);
                                }

                                if !handle.is_null() {
                                    if pass == 0 {
                                        let mut pmc: PROCESS_MEMORY_COUNTERS_EX =
                                            std::mem::zeroed();
                                        pmc.cb =
                                            std::mem::size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32;
                                        if K32GetProcessMemoryInfo(
                                            handle,
                                            &mut pmc as *mut _ as *mut PROCESS_MEMORY_COUNTERS,
                                            pmc.cb,
                                        ) != 0
                                        {
                                            let entry_stat = total_ws_before
                                                .entry(raw_name.clone())
                                                .or_insert((0, 0));
                                            entry_stat.0 += 1;
                                            entry_stat.1 += pmc.WorkingSetSize as u64;
                                        }
                                    }

                                    let mut ok = false;
                                    // Force hard minimum working set floor off so heavy apps (Browsers/Electron/Games) drop all idle pages
                                    let flags = if depth == MemoryCleanDepth::Light {
                                        0
                                    } else {
                                        QUOTA_LIMITS_HARDWS_MIN_DISABLE
                                            | QUOTA_LIMITS_HARDWS_MAX_ENABLE
                                    };
                                    if SetProcessWorkingSetSizeEx(
                                        handle,
                                        usize::MAX,
                                        usize::MAX,
                                        flags,
                                    ) != 0
                                    {
                                        ok = true;
                                    }
                                    if K32EmptyWorkingSet(handle) != 0 {
                                        ok = true;
                                    }

                                    if ok && pass == 0 {
                                        processes_trimmed += 1;
                                        areas.working_sets_cleaned = true;
                                    }

                                    CloseHandle(handle);
                                }
                            }
                        }

                        if Process32NextW(snap, &mut entry) == 0 {
                            break;
                        }
                    }
                }
                CloseHandle(snap);
            }

            // On Medium and Deep modes, invoke NT Kernel memory reduction stages between passes
            if pass == 0 && depth != MemoryCleanDepth::Light {
                // Stage A: Kernel-wide Working Set Empty (SystemMemoryListInformation = 80, cmd = 2)
                let mut cmd_empty_ws: i32 = MEMORY_EMPTY_WORKING_SETS;
                if NtSetSystemInformation(
                    SYSTEM_MEMORY_LIST_INFORMATION,
                    &mut cmd_empty_ws as *mut _ as *mut std::ffi::c_void,
                    std::mem::size_of::<i32>() as u32,
                ) >= 0
                {
                    areas.working_sets_cleaned = true;
                }

                // Stage B: Flush Windows System File Cache Working Set
                if SetSystemFileCacheSize(usize::MAX, usize::MAX, 0) != 0 {
                    areas.system_file_cache_flushed = true;
                }
                let mut sfc_info = SystemFileCacheInformation {
                    current_size: 0,
                    peak_size: 0,
                    page_fault_count: 0,
                    minimum_working_set: usize::MAX,
                    maximum_working_set: usize::MAX,
                    current_size_including_transition_in_pages: 0,
                    peak_size_including_transition_in_pages: 0,
                    transition_repurpose_count: 0,
                    flags: 0,
                };
                if NtSetSystemInformation(
                    SYSTEM_FILECACHE_INFORMATION_EX,
                    &mut sfc_info as *mut _ as *mut std::ffi::c_void,
                    std::mem::size_of::<SystemFileCacheInformation>() as u32,
                ) >= 0
                {
                    areas.system_file_cache_flushed = true;
                }

                // Stage C: Flush Registry Cache in RAM (Windows 8.1 / 10 / 11)
                if NtSetSystemInformation(
                    SYSTEM_REGISTRY_RECONCILIATION_INFORMATION,
                    std::ptr::null_mut(),
                    0,
                ) >= 0
                {
                    areas.registry_cache_flushed = true;
                }

                // Stage D: Purge Low-Priority Standby List (Medium + Deep)
                let mut cmd_low_standby: i32 = MEMORY_PURGE_LOW_PRIORITY_STANDBY_LIST;
                if NtSetSystemInformation(
                    SYSTEM_MEMORY_LIST_INFORMATION,
                    &mut cmd_low_standby as *mut _ as *mut std::ffi::c_void,
                    std::mem::size_of::<i32>() as u32,
                ) >= 0
                {
                    areas.low_priority_standby_purged = true;
                }

                // Deep Mode Extra Kernel Stages: Combine Physical Memory + Flush Modified Page List + Purge Full Standby List
                if depth == MemoryCleanDepth::Deep {
                    let mut combine_info = MemoryCombineInformationEx {
                        handle: 0,
                        pages_combined: 0,
                        flags: 0,
                    };
                    if NtSetSystemInformation(
                        SYSTEM_COMBINE_PHYSICAL_MEMORY_INFORMATION,
                        &mut combine_info as *mut _ as *mut std::ffi::c_void,
                        std::mem::size_of::<MemoryCombineInformationEx>() as u32,
                    ) >= 0
                    {
                        areas.physical_memory_combined = true;
                        areas.pages_combined_count = combine_info.pages_combined as u64;
                    }

                    let mut cmd_flush_mod: i32 = MEMORY_FLUSH_MODIFIED_LIST;
                    if NtSetSystemInformation(
                        SYSTEM_MEMORY_LIST_INFORMATION,
                        &mut cmd_flush_mod as *mut _ as *mut std::ffi::c_void,
                        std::mem::size_of::<i32>() as u32,
                    ) >= 0
                    {
                        areas.modified_page_list_flushed = true;
                    }

                    let mut cmd_standby: i32 = MEMORY_PURGE_STANDBY_LIST;
                    if NtSetSystemInformation(
                        SYSTEM_MEMORY_LIST_INFORMATION,
                        &mut cmd_standby as *mut _ as *mut std::ffi::c_void,
                        std::mem::size_of::<i32>() as u32,
                    ) >= 0
                    {
                        areas.standby_list_purged = true;
                    }
                }
            }
        }

        // Measure post-trim working sets across processes to compute per-process savings
        let snap_after = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snap_after != INVALID_HANDLE_VALUE && !snap_after.is_null() {
            let mut entry: PROCESSENTRY32W = std::mem::zeroed();
            entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
            if Process32FirstW(snap_after, &mut entry) != 0 {
                loop {
                    let pid = entry.th32ProcessID;
                    if pid > 4 {
                        let raw_name = String::from_utf16_lossy(&entry.szExeFile)
                            .trim_end_matches('\0')
                            .to_string();
                        if total_ws_before.contains_key(&raw_name) {
                            let mut handle =
                                OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
                            if handle.is_null() {
                                handle = OpenProcess(PROCESS_QUERY_INFORMATION, 0, pid);
                            }
                            if !handle.is_null() {
                                let mut pmc: PROCESS_MEMORY_COUNTERS_EX = std::mem::zeroed();
                                pmc.cb =
                                    std::mem::size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32;
                                if K32GetProcessMemoryInfo(
                                    handle,
                                    &mut pmc as *mut _ as *mut PROCESS_MEMORY_COUNTERS,
                                    pmc.cb,
                                ) != 0
                                {
                                    *total_ws_after.entry(raw_name).or_insert(0) +=
                                        pmc.WorkingSetSize as u64;
                                }
                                CloseHandle(handle);
                            }
                        }
                    }
                    if Process32NextW(snap_after, &mut entry) == 0 {
                        break;
                    }
                }
            }
            CloseHandle(snap_after);
        }
    }

    let mut top_freed_processes: Vec<ProcessTrimDiff> = Vec::new();
    let mut working_set_freed_bytes = 0u64;

    for (name, (count, before_b)) in &total_ws_before {
        let after_b = *total_ws_after.get(name).unwrap_or(before_b);
        if *before_b > after_b {
            let diff = before_b.saturating_sub(after_b);
            working_set_freed_bytes += diff;
            if diff >= 512 * 1024 {
                top_freed_processes.push(ProcessTrimDiff {
                    name: name.clone(),
                    instance_count: *count,
                    before_bytes: *before_b,
                    after_bytes: after_b,
                    freed_bytes: diff,
                });
            }
        }
    }
    top_freed_processes.sort_by(|a, b| b.freed_bytes.cmp(&a.freed_bytes));
    top_freed_processes.truncate(10);

    let after = get_memory_snapshot();
    let physical_avail_gain = after
        .avail_physical_bytes
        .saturating_sub(before.avail_physical_bytes);
    let cache_freed = before
        .system_cache_bytes
        .saturating_sub(after.system_cache_bytes);

    // Total freed is the larger of the direct physical availability jump or the measured working set reduction
    let bytes_freed = physical_avail_gain.max(working_set_freed_bytes);

    Ok(MemoryCleanReport {
        depth: depth.as_str().to_string(),
        before,
        after,
        bytes_freed,
        working_set_freed_bytes,
        standby_and_cache_freed_bytes: cache_freed,
        processes_scanned,
        processes_trimmed,
        areas,
        top_freed_processes,
        duration_ms: start.elapsed().as_millis() as u64,
        completed_at: Utc::now().to_rfc3339(),
        trigger_source: trigger_source.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_engine_abi_and_snapshot() {
        assert_eq!(memory_engine_abi_version(), SALEEM_ABI_VERSION);
        let snap = get_memory_snapshot();
        assert!(snap.total_physical_bytes > 0);
        assert!(snap.process_count > 0);
    }

    #[test]
    fn test_top_processes_and_reduction() {
        let top = get_top_memory_processes(10);
        assert!(!top.is_empty());
        let report = execute_memory_reduction(MemoryCleanDepth::Light).expect("Should succeed");
        assert_eq!(report.depth, "Light");
        assert!(report.processes_scanned > 0);
    }
}
