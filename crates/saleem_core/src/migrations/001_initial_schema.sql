-- SALEEM Initial Database Schema (Migration 001)
-- Matches Section 15 of MASTER_PROJECT_SPECIFICATION.md

CREATE TABLE IF NOT EXISTS scans (
    id TEXT PRIMARY KEY NOT NULL,
    scan_type TEXT NOT NULL,
    target TEXT NOT NULL,
    started_at TEXT NOT NULL,
    completed_at TEXT,
    status TEXT NOT NULL,
    files_seen INTEGER NOT NULL DEFAULT 0,
    folders_seen INTEGER NOT NULL DEFAULT 0,
    errors_count INTEGER NOT NULL DEFAULT 0,
    result_summary TEXT
);

CREATE TABLE IF NOT EXISTS file_entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    scan_id TEXT NOT NULL,
    path TEXT NOT NULL,
    file_name TEXT NOT NULL,
    extension TEXT NOT NULL,
    logical_size INTEGER NOT NULL,
    allocated_size INTEGER NOT NULL,
    modified_at TEXT NOT NULL,
    attributes INTEGER NOT NULL DEFAULT 0,
    content_hash TEXT,
    last_verified_at TEXT NOT NULL,
    FOREIGN KEY (scan_id) REFERENCES scans(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_file_entries_scan_id ON file_entries(scan_id);
CREATE INDEX IF NOT EXISTS idx_file_entries_logical_size ON file_entries(logical_size DESC);
CREATE INDEX IF NOT EXISTS idx_file_entries_extension ON file_entries(extension);

CREATE TABLE IF NOT EXISTS cleanup_rules (
    id TEXT PRIMARY KEY NOT NULL,
    version TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    risk_level TEXT NOT NULL,
    rule_metadata TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS cleanup_jobs (
    id TEXT PRIMARY KEY NOT NULL,
    started_at TEXT NOT NULL,
    completed_at TEXT,
    status TEXT NOT NULL,
    total_candidates INTEGER NOT NULL DEFAULT 0,
    succeeded INTEGER NOT NULL DEFAULT 0,
    failed INTEGER NOT NULL DEFAULT 0,
    bytes_reclaimed INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS cleanup_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    job_id TEXT NOT NULL,
    original_path TEXT NOT NULL,
    action TEXT NOT NULL,
    result TEXT NOT NULL,
    reason TEXT NOT NULL,
    original_hash TEXT,
    backup_reference TEXT,
    FOREIGN KEY (job_id) REFERENCES cleanup_jobs(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_cleanup_items_job_id ON cleanup_items(job_id);

CREATE TABLE IF NOT EXISTS security_scans (
    id TEXT PRIMARY KEY NOT NULL,
    target TEXT NOT NULL,
    started_at TEXT NOT NULL,
    completed_at TEXT,
    status TEXT NOT NULL,
    engine_version TEXT NOT NULL,
    rules_version TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS security_findings (
    id TEXT PRIMARY KEY NOT NULL,
    scan_id TEXT NOT NULL,
    path TEXT NOT NULL,
    sha256 TEXT NOT NULL,
    risk_level TEXT NOT NULL,
    confidence TEXT NOT NULL,
    reasons_json TEXT NOT NULL,
    signature_status TEXT NOT NULL,
    recommendation TEXT NOT NULL,
    FOREIGN KEY (scan_id) REFERENCES security_scans(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_security_findings_scan_id ON security_findings(scan_id);
CREATE INDEX IF NOT EXISTS idx_security_findings_risk ON security_findings(risk_level);

CREATE TABLE IF NOT EXISTS quarantine_items (
    id TEXT PRIMARY KEY NOT NULL,
    stored_name TEXT NOT NULL UNIQUE,
    original_path TEXT NOT NULL,
    sha256 TEXT NOT NULL,
    size INTEGER NOT NULL,
    quarantined_at TEXT NOT NULL,
    reason TEXT NOT NULL,
    status TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS audit_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL,
    event_type TEXT NOT NULL,
    subject TEXT NOT NULL,
    result TEXT NOT NULL,
    details_redacted TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_audit_events_timestamp ON audit_events(timestamp DESC);
