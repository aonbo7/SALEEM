use crate::error::SaleemResult;
use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;

const MIGRATION_001_SQL: &str = include_str!("migrations/001_initial_schema.sql");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEventRecord {
    pub id: i64,
    pub timestamp: String,
    pub event_type: String,
    pub subject: String,
    pub result: String,
    pub details_redacted: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseHealthSummary {
    pub schema_version: u32,
    pub tables_count: usize,
    pub scans_count: u64,
    pub cleanup_jobs_count: u64,
    pub security_scans_count: u64,
    pub quarantine_active_count: u64,
    pub audit_events_count: u64,
}

/// Opens a SQLite database at the given path (or in-memory), configures WAL mode and foreign keys,
/// and applies all pending migrations atomically.
pub fn init_database(db_path: &Path) -> SaleemResult<Connection> {
    if let Some(parent) = db_path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }
    let mut conn = Connection::open(db_path)?;
    configure_and_migrate(&mut conn)?;
    Ok(conn)
}

pub fn init_in_memory_database() -> SaleemResult<Connection> {
    let mut conn = Connection::open_in_memory()?;
    configure_and_migrate(&mut conn)?;
    Ok(conn)
}

fn configure_and_migrate(conn: &mut Connection) -> SaleemResult<()> {
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA foreign_keys = ON;
         CREATE TABLE IF NOT EXISTS schema_migrations (
             version INTEGER PRIMARY KEY NOT NULL,
             applied_at TEXT NOT NULL
         );",
    )?;

    let current_version: u32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if current_version < 1 {
        let tx = conn.transaction()?;
        tx.execute_batch(MIGRATION_001_SQL)?;
        let now = Utc::now().to_rfc3339();
        tx.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, ?2)",
            params![1u32, now],
        )?;
        // Seed default settings if empty
        tx.execute(
            "INSERT OR IGNORE INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3)",
            params!["ui.language", "ar", &now],
        )?;
        tx.execute(
            "INSERT OR IGNORE INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3)",
            params!["ui.theme", "dark", &now],
        )?;
        tx.execute(
            "INSERT OR IGNORE INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3)",
            params!["scan.max_file_entries_per_scan", "5000", &now],
        )?;
        tx.commit()?;
    }

    Ok(())
}

/// Appends a redacted audit event to the `audit_events` table.
pub fn record_audit_event(
    conn: &Connection,
    event_type: &str,
    subject: &str,
    result: &str,
    details_redacted: &str,
) -> SaleemResult<i64> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO audit_events (timestamp, event_type, subject, result, details_redacted)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![now, event_type, subject, result, details_redacted],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Lists recent audit events in reverse chronological order.
pub fn list_recent_audit_events(
    conn: &Connection,
    limit: usize,
) -> SaleemResult<Vec<AuditEventRecord>> {
    let mut stmt = conn.prepare(
        "SELECT id, timestamp, event_type, subject, result, details_redacted
         FROM audit_events
         ORDER BY id DESC
         LIMIT ?1",
    )?;
    let rows = stmt.query_map(params![limit as i64], |row| {
        Ok(AuditEventRecord {
            id: row.get(0)?,
            timestamp: row.get(1)?,
            event_type: row.get(2)?,
            subject: row.get(3)?,
            result: row.get(4)?,
            details_redacted: row.get(5)?,
        })
    })?;

    let mut events = Vec::new();
    for item in rows {
        events.push(item?);
    }
    Ok(events)
}

/// Reads a setting value by key.
pub fn get_setting(conn: &Connection, key: &str) -> SaleemResult<Option<String>> {
    let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
    let mut rows = stmt.query(params![key])?;
    if let Some(row) = rows.next()? {
        Ok(Some(row.get(0)?))
    } else {
        Ok(None)
    }
}

/// Sets or updates a setting value by key.
pub fn set_setting(conn: &Connection, key: &str, value: &str) -> SaleemResult<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
        params![key, value, now],
    )?;
    Ok(())
}

/// Returns a real health and statistics summary of the SQLite database.
pub fn get_database_health_summary(conn: &Connection) -> SaleemResult<DatabaseHealthSummary> {
    let schema_version: u32 = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
        [],
        |r| r.get(0),
    )?;
    let tables_count: usize = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
        [],
        |r| r.get::<_, i64>(0).map(|v| v as usize),
    )?;
    let scans_count: u64 =
        conn.query_row("SELECT COUNT(*) FROM scans", [], |r| r.get::<_, i64>(0))? as u64;
    let cleanup_jobs_count: u64 = conn.query_row("SELECT COUNT(*) FROM cleanup_jobs", [], |r| {
        r.get::<_, i64>(0)
    })? as u64;
    let security_scans_count: u64 =
        conn.query_row("SELECT COUNT(*) FROM security_scans", [], |r| {
            r.get::<_, i64>(0)
        })? as u64;
    let quarantine_active_count: u64 = conn.query_row(
        "SELECT COUNT(*) FROM quarantine_items WHERE status = 'QUARANTINED'",
        [],
        |r| r.get::<_, i64>(0),
    )? as u64;
    let audit_events_count: u64 = conn.query_row("SELECT COUNT(*) FROM audit_events", [], |r| {
        r.get::<_, i64>(0)
    })? as u64;

    Ok(DatabaseHealthSummary {
        schema_version,
        tables_count,
        scans_count,
        cleanup_jobs_count,
        security_scans_count,
        quarantine_active_count,
        audit_events_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrations_create_all_required_tables_idempotently() {
        let mut conn = init_in_memory_database().expect("Failed to init in-memory DB");
        // Re-running migration must be idempotent
        configure_and_migrate(&mut conn).expect("Re-migration failed");

        let summary = get_database_health_summary(&conn).expect("Health summary failed");
        assert_eq!(summary.schema_version, 1);
        // 10 specification tables + schema_migrations = 11
        assert_eq!(summary.tables_count, 11);

        let lang = get_setting(&conn, "ui.language").expect("get_setting failed");
        assert_eq!(lang.as_deref(), Some("ar"));
    }

    #[test]
    fn test_audit_event_logging_and_retrieval() {
        let conn = init_in_memory_database().unwrap();
        let row_id = record_audit_event(
            &conn,
            "SYSTEM_STARTUP",
            "SALEEM.Core",
            "SUCCESS",
            "Database initialized in WAL mode",
        )
        .unwrap();
        assert!(row_id > 0);

        let events = list_recent_audit_events(&conn, 10).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "SYSTEM_STARTUP");
        assert_eq!(events[0].result, "SUCCESS");
    }
}
