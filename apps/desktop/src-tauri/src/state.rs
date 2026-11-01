use chrono::Utc;
use rusqlite::params;
use saleem_cleanup_engine::{default_cleanup_rules, CleanupExecutionReport, CleanupPlanResult};
use saleem_core::db::{init_database, record_audit_event};
use saleem_core::{JobRegistry, SaleemError, SaleemResult};
use saleem_disk_engine::{
    DiskScanResult, DuplicateRemovalGroupSelection, DuplicateRemovalReport, DuplicateScanResult,
};
use saleem_inventory::{InventoryScanResult, LeftoverRemovalReport};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct AppState {
    pub db_path: PathBuf,
    pub db: Arc<Mutex<rusqlite::Connection>>,
    pub job_registry: JobRegistry,
    pub scan_results: Arc<Mutex<HashMap<String, DiskScanResult>>>,
    pub latest_scan_job_id: Arc<Mutex<Option<String>>>,
    pub duplicate_results: Arc<Mutex<HashMap<String, DuplicateScanResult>>>,
    pub latest_duplicate_job_id: Arc<Mutex<Option<String>>>,
    pub cleanup_plans: Arc<Mutex<HashMap<String, CleanupPlanResult>>>,
    pub latest_cleanup_job_id: Arc<Mutex<Option<String>>>,
    pub inventory_results: Arc<Mutex<HashMap<String, InventoryScanResult>>>,
    pub latest_inventory_job_id: Arc<Mutex<Option<String>>>,
}

impl AppState {
    pub fn initialize(app_data_dir: PathBuf) -> SaleemResult<Self> {
        std::fs::create_dir_all(&app_data_dir)?;
        let db_path = app_data_dir.join("saleem_state.sqlite3");
        let conn = init_database(&db_path)?;
        let _ = record_audit_event(
            &conn,
            "APP_INIT",
            "SALEEM.Core",
            "SUCCESS",
            "SALEEM desktop state and SQLite WAL database initialized",
        );
        Ok(Self {
            db_path,
            db: Arc::new(Mutex::new(conn)),
            job_registry: JobRegistry::new(),
            scan_results: Arc::new(Mutex::new(HashMap::new())),
            latest_scan_job_id: Arc::new(Mutex::new(None)),
            duplicate_results: Arc::new(Mutex::new(HashMap::new())),
            latest_duplicate_job_id: Arc::new(Mutex::new(None)),
            cleanup_plans: Arc::new(Mutex::new(HashMap::new())),
            latest_cleanup_job_id: Arc::new(Mutex::new(None)),
            inventory_results: Arc::new(Mutex::new(HashMap::new())),
            latest_inventory_job_id: Arc::new(Mutex::new(None)),
        })
    }

    pub fn with_db<F, T>(&self, f: F) -> SaleemResult<T>
    where
        F: FnOnce(&mut rusqlite::Connection) -> SaleemResult<T>,
    {
        let mut guard = self
            .db
            .lock()
            .map_err(|_| SaleemError::DatabaseError("Database mutex poisoned".to_string()))?;
        f(&mut guard)
    }

    /// Persists a completed DiskScanResult into SQLite `scans`, `file_entries` (top entries), and `audit_events`.
    pub fn persist_disk_scan(&self, result: &DiskScanResult) -> SaleemResult<()> {
        self.with_db(|conn| {
            let tx = conn.transaction()?;
            let summary_json = serde_json::json!({
                "total_logical_bytes": result.total_logical_bytes,
                "total_allocated_bytes": result.total_allocated_bytes,
                "duration_ms": result.duration_ms,
                "is_partial": result.is_partial,
                "reparse_points_skipped": result.reparse_points_skipped,
            })
            .to_string();

            tx.execute(
                "INSERT INTO scans (id, scan_type, target, started_at, completed_at, status, files_seen, folders_seen, errors_count, result_summary)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    result.scan_id,
                    "DISK_ANALYZER",
                    result.target_path,
                    result.started_at,
                    result.completed_at,
                    result.status,
                    result.files_count as i64,
                    result.folders_count as i64,
                    result.errors_count as i64,
                    summary_json,
                ],
            )?;

            for entry in result.top_files.iter().take(100) {
                tx.execute(
                    "INSERT INTO file_entries (scan_id, path, file_name, extension, logical_size, allocated_size, modified_at, attributes, content_hash, last_verified_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0, NULL, ?8)",
                    params![
                        result.scan_id,
                        entry.path,
                        entry.file_name,
                        entry.extension,
                        entry.logical_size as i64,
                        entry.allocated_size as i64,
                        entry.modified_at,
                        result.completed_at,
                    ],
                )?;
            }

            tx.commit()?;

            let _ = record_audit_event(
                conn,
                "DISK_SCAN_COMPLETED",
                "SALEEM.DiskEngine",
                &result.status,
                &format!(
                    "target={}, files={}, folders={}, logical_bytes={}, errors={}",
                    result.target_path,
                    result.files_count,
                    result.folders_count,
                    result.total_logical_bytes,
                    result.errors_count
                ),
            );
            Ok(())
        })
    }

    /// Persists a completed DuplicateScanResult into SQLite `scans`, `file_entries` (for duplicate files), and `audit_events`.
    pub fn persist_duplicate_scan(&self, result: &DuplicateScanResult) -> SaleemResult<()> {
        self.with_db(|conn| {
            let tx = conn.transaction()?;
            let summary_json = serde_json::json!({
                "total_duplicate_groups": result.total_duplicate_groups,
                "total_duplicate_files": result.total_duplicate_files,
                "total_reclaimable_bytes": result.total_reclaimable_bytes,
                "candidate_size_groups": result.candidate_size_groups,
                "partial_hashes_computed": result.partial_hashes_computed,
                "full_hashes_computed": result.full_hashes_computed,
                "hardlinks_flagged": result.hardlinks_flagged,
                "duration_ms": result.duration_ms,
            })
            .to_string();

            tx.execute(
                "INSERT INTO scans (id, scan_type, target, started_at, completed_at, status, files_seen, folders_seen, errors_count, result_summary)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0, ?8, ?9)",
                params![
                    result.scan_id,
                    "DUPLICATE_FINDER",
                    result.target_path,
                    result.started_at,
                    result.completed_at,
                    result.status,
                    result.files_evaluated as i64,
                    result.errors_count as i64,
                    summary_json,
                ],
            )?;

            for group in result.groups.iter().take(100) {
                for member in &group.copies {
                    tx.execute(
                        "INSERT INTO file_entries (scan_id, path, file_name, extension, logical_size, allocated_size, modified_at, attributes, content_hash, last_verified_at)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0, ?8, ?9)",
                        params![
                            result.scan_id,
                            member.path,
                            member.file_name,
                            group.extension,
                            group.file_size_bytes as i64,
                            group.file_size_bytes as i64,
                            member.modified_at,
                            group.sha256,
                            result.completed_at,
                        ],
                    )?;
                }
            }

            tx.commit()?;

            let _ = record_audit_event(
                conn,
                "DUPLICATE_SCAN_COMPLETED",
                "SALEEM.DiskEngine",
                &result.status,
                &format!(
                    "target={}, groups={}, reclaimable_bytes={}, full_hashes={}",
                    result.target_path,
                    result.total_duplicate_groups,
                    result.total_reclaimable_bytes,
                    result.full_hashes_computed
                ),
            );
            Ok(())
        })
    }

    /// Persists a completed DuplicateRemovalReport into SQLite `cleanup_jobs`, `cleanup_items`, and `audit_events`.
    pub fn persist_duplicate_removal(
        &self,
        selections: &[DuplicateRemovalGroupSelection],
        report: &DuplicateRemovalReport,
    ) -> SaleemResult<()> {
        self.with_db(|conn| {
            let now = Utc::now().to_rfc3339();
            let status = if report.failed_count == 0 {
                "COMPLETED"
            } else if report.succeeded_count > 0 {
                "PARTIAL"
            } else {
                "FAILED"
            };

            let tx = conn.transaction()?;
            tx.execute(
                "INSERT INTO cleanup_jobs (id, started_at, completed_at, status, total_candidates, succeeded, failed, bytes_reclaimed)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    report.job_id,
                    &now,
                    &now,
                    status,
                    (report.succeeded_count + report.failed_count) as i64,
                    report.succeeded_count as i64,
                    report.failed_count as i64,
                    report.bytes_reclaimed as i64,
                ],
            )?;

            for sel in selections {
                for del_path in &sel.delete_paths {
                    let removed = report.deleted_paths.iter().any(|p| p == del_path);
                    let result_str = if removed { "REMOVED" } else { "SKIPPED_OR_FAILED" };
                    tx.execute(
                        "INSERT INTO cleanup_items (job_id, original_path, action, result, reason, original_hash, backup_reference)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, NULL)",
                        params![
                            report.job_id,
                            del_path,
                            "DELETE_DUPLICATE",
                            result_str,
                            format!("Retained copy: {}", sel.retained_reference_path),
                            sel.group_sha256,
                        ],
                    )?;
                }
            }

            tx.commit()?;

            let _ = record_audit_event(
                conn,
                "DUPLICATE_REMOVAL_EXECUTED",
                "SALEEM.DiskEngine",
                status,
                &format!(
                    "removed={}, failed={}, bytes_reclaimed={}",
                    report.succeeded_count, report.failed_count, report.bytes_reclaimed
                ),
            );
            Ok(())
        })
    }

    /// Persists a CleanupPlanResult and syncs `cleanup_rules` into SQLite.
    pub fn persist_cleanup_plan(&self, plan: &CleanupPlanResult) -> SaleemResult<()> {
        self.with_db(|conn| {
            let tx = conn.transaction()?;

            for rule in default_cleanup_rules() {
                let meta_json = serde_json::to_string(&rule).unwrap_or_default();
                tx.execute(
                    "INSERT OR REPLACE INTO cleanup_rules (id, version, enabled, risk_level, rule_metadata)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![
                        rule.id,
                        "2026.10.1",
                        if rule.enabled_by_default { 1i64 } else { 0i64 },
                        rule.risk_level,
                        meta_json,
                    ],
                )?;
            }

            let summary_json = serde_json::json!({
                "total_candidates": plan.total_candidates,
                "total_locked_skipped": plan.total_locked_skipped,
                "total_reclaimable_bytes": plan.total_reclaimable_bytes,
                "duration_ms": plan.duration_ms,
            })
            .to_string();

            tx.execute(
                "INSERT INTO scans (id, scan_type, target, started_at, completed_at, status, files_seen, folders_seen, errors_count, result_summary)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0, ?8, ?9)",
                params![
                    plan.plan_id,
                    "SAFE_CLEANUP_PLAN",
                    "User Temp & Cache Scopes",
                    plan.started_at,
                    plan.completed_at,
                    plan.pipeline_stage,
                    plan.total_candidates as i64,
                    plan.errors_count as i64,
                    summary_json,
                ],
            )?;

            tx.commit()?;

            let _ = record_audit_event(
                conn,
                "CLEANUP_PLAN_GENERATED",
                "SALEEM.CleanupEngine",
                &plan.pipeline_stage,
                &format!(
                    "plan_id={}, candidates={}, reclaimable_bytes={}, locked_skipped={}",
                    plan.plan_id,
                    plan.total_candidates,
                    plan.total_reclaimable_bytes,
                    plan.total_locked_skipped
                ),
            );
            Ok(())
        })
    }

    /// Persists a completed CleanupExecutionReport into SQLite `cleanup_jobs`, `cleanup_items`, and `audit_events`.
    pub fn persist_cleanup_execution(&self, report: &CleanupExecutionReport) -> SaleemResult<()> {
        self.with_db(|conn| {
            let status = if report.skipped_or_failed_count == 0 {
                "COMPLETED"
            } else if report.succeeded_count > 0 {
                "PARTIAL"
            } else {
                "FAILED"
            };

            let tx = conn.transaction()?;
            tx.execute(
                "INSERT INTO cleanup_jobs (id, started_at, completed_at, status, total_candidates, succeeded, failed, bytes_reclaimed)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    report.job_id,
                    report.started_at,
                    report.completed_at,
                    status,
                    report.total_requested as i64,
                    report.succeeded_count as i64,
                    report.skipped_or_failed_count as i64,
                    report.bytes_reclaimed as i64,
                ],
            )?;

            for outcome in &report.outcomes {
                tx.execute(
                    "INSERT INTO cleanup_items (job_id, original_path, action, result, reason, original_hash, backup_reference)
                     VALUES (?1, ?2, ?3, ?4, ?5, NULL, NULL)",
                    params![
                        report.job_id,
                        outcome.path,
                        outcome.action,
                        outcome.status,
                        format!("[{}] {}", outcome.rule_id, outcome.detail),
                    ],
                )?;
            }

            tx.commit()?;

            let _ = record_audit_event(
                conn,
                "CLEANUP_EXECUTION_COMPLETED",
                "SALEEM.CleanupEngine",
                status,
                &format!(
                    "job_id={}, succeeded={}, skipped_or_failed={}, bytes_reclaimed={}",
                    report.job_id,
                    report.succeeded_count,
                    report.skipped_or_failed_count,
                    report.bytes_reclaimed
                ),
            );
            Ok(())
        })
    }

    /// Persists an InventoryScanResult into SQLite `scans` and `audit_events`.
    pub fn persist_inventory_scan(&self, result: &InventoryScanResult) -> SaleemResult<()> {
        self.with_db(|conn| {
            let summary_json = serde_json::json!({
                "total_apps_count": result.total_apps_count,
                "total_games_count": result.total_games_count,
                "total_leftovers_count": result.total_leftovers_count,
                "high_confidence_leftovers_count": result.high_confidence_leftovers_count,
                "total_leftover_bytes": result.total_leftover_bytes,
                "high_confidence_reclaimable_bytes": result.high_confidence_reclaimable_bytes,
                "duration_ms": result.duration_ms,
            })
            .to_string();

            conn.execute(
                "INSERT INTO scans (id, scan_type, target, started_at, completed_at, status, files_seen, folders_seen, errors_count, result_summary)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0, ?9)",
                params![
                    result.scan_id,
                    "APP_AND_LEFTOVER_INVENTORY",
                    "Windows Registry + Launchers + AppData",
                    result.started_at,
                    result.completed_at,
                    "COMPLETED",
                    (result.total_apps_count + result.total_games_count) as i64,
                    result.total_leftovers_count as i64,
                    summary_json,
                ],
            )?;

            let _ = record_audit_event(
                conn,
                "INVENTORY_SCAN_COMPLETED",
                "SALEEM.Inventory",
                "COMPLETED",
                &format!(
                    "apps={}, games={}, leftovers={}, high_confidence={}, leftover_bytes={}",
                    result.total_apps_count,
                    result.total_games_count,
                    result.total_leftovers_count,
                    result.high_confidence_leftovers_count,
                    result.total_leftover_bytes
                ),
            );
            Ok(())
        })
    }

    /// Persists a LeftoverRemovalReport into SQLite `cleanup_jobs`, `cleanup_items`, and `audit_events`.
    pub fn persist_leftover_removal(&self, report: &LeftoverRemovalReport) -> SaleemResult<()> {
        self.with_db(|conn| {
            let status = if report.skipped_or_failed_count == 0 {
                "COMPLETED"
            } else if report.succeeded_count > 0 {
                "PARTIAL"
            } else {
                "FAILED"
            };

            let tx = conn.transaction()?;
            tx.execute(
                "INSERT INTO cleanup_jobs (id, started_at, completed_at, status, total_candidates, succeeded, failed, bytes_reclaimed)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    report.job_id,
                    report.started_at,
                    report.completed_at,
                    status,
                    report.total_requested as i64,
                    report.succeeded_count as i64,
                    report.skipped_or_failed_count as i64,
                    report.bytes_reclaimed as i64,
                ],
            )?;

            for outcome in &report.outcomes {
                tx.execute(
                    "INSERT INTO cleanup_items (job_id, original_path, action, result, reason, original_hash, backup_reference)
                     VALUES (?1, ?2, ?3, ?4, ?5, NULL, NULL)",
                    params![
                        report.job_id,
                        outcome.path,
                        "DELETE_LEFTOVER_FOLDER",
                        outcome.status,
                        outcome.reason,
                    ],
                )?;
            }

            tx.commit()?;

            let _ = record_audit_event(
                conn,
                "LEFTOVER_REMOVAL_COMPLETED",
                "SALEEM.Inventory",
                status,
                &format!(
                    "job_id={}, succeeded={}, skipped_or_failed={}, bytes_reclaimed={}",
                    report.job_id,
                    report.succeeded_count,
                    report.skipped_or_failed_count,
                    report.bytes_reclaimed
                ),
            );
            Ok(())
        })
    }
}
