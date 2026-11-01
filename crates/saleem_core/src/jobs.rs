use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use uuid::Uuid;

/// Thread-safe cancellation token shared with background or FFI scan workers.
#[derive(Clone, Default)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }

    pub fn as_atomic_ptr(&self) -> *const AtomicBool {
        Arc::as_ptr(&self.cancelled)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobStatus {
    Running,
    Completed,
    Cancelled,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobProgressSnapshot {
    pub job_id: String,
    pub job_type: String,
    pub target: String,
    pub status: JobStatus,
    pub files_processed: u64,
    pub folders_processed: u64,
    pub bytes_processed: u64,
    pub current_path: String,
    pub errors_count: u64,
}

struct JobEntry {
    token: CancellationToken,
    snapshot: JobProgressSnapshot,
}

/// Central registry for active and recently completed background jobs.
#[derive(Clone, Default)]
pub struct JobRegistry {
    inner: Arc<Mutex<HashMap<String, JobEntry>>>,
}

impl JobRegistry {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn create_job(&self, job_type: &str, target: &str) -> (String, CancellationToken) {
        let job_id = Uuid::new_v4().to_string();
        let token = CancellationToken::new();
        let snapshot = JobProgressSnapshot {
            job_id: job_id.clone(),
            job_type: job_type.to_string(),
            target: target.to_string(),
            status: JobStatus::Running,
            files_processed: 0,
            folders_processed: 0,
            bytes_processed: 0,
            current_path: String::new(),
            errors_count: 0,
        };
        if let Ok(mut map) = self.inner.lock() {
            map.insert(
                job_id.clone(),
                JobEntry {
                    token: token.clone(),
                    snapshot,
                },
            );
        }
        (job_id, token)
    }

    pub fn update_progress(
        &self,
        job_id: &str,
        files_processed: u64,
        folders_processed: u64,
        bytes_processed: u64,
        current_path: &str,
        errors_count: u64,
    ) {
        if let Ok(mut map) = self.inner.lock() {
            if let Some(entry) = map.get_mut(job_id) {
                entry.snapshot.files_processed = files_processed;
                entry.snapshot.folders_processed = folders_processed;
                entry.snapshot.bytes_processed = bytes_processed;
                entry.snapshot.current_path = current_path.to_string();
                entry.snapshot.errors_count = errors_count;
            }
        }
    }

    pub fn finish_job(&self, job_id: &str, status: JobStatus) {
        if let Ok(mut map) = self.inner.lock() {
            if let Some(entry) = map.get_mut(job_id) {
                entry.snapshot.status = status;
            }
        }
    }

    pub fn cancel_job(&self, job_id: &str) -> bool {
        if let Ok(mut map) = self.inner.lock() {
            if let Some(entry) = map.get_mut(job_id) {
                entry.token.cancel();
                entry.snapshot.status = JobStatus::Cancelled;
                return true;
            }
        }
        false
    }

    pub fn get_snapshot(&self, job_id: &str) -> Option<JobProgressSnapshot> {
        self.inner
            .lock()
            .ok()
            .and_then(|map| map.get(job_id).map(|e| e.snapshot.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_creation_and_cancellation() {
        let registry = JobRegistry::new();
        let (job_id, token) = registry.create_job("DISK_SCAN", r"C:\Users");
        assert!(!token.is_cancelled());

        let snap = registry.get_snapshot(&job_id).unwrap();
        assert_eq!(snap.status, JobStatus::Running);

        assert!(registry.cancel_job(&job_id));
        assert!(token.is_cancelled());

        let snap_after = registry.get_snapshot(&job_id).unwrap();
        assert_eq!(snap_after.status, JobStatus::Cancelled);
    }
}
