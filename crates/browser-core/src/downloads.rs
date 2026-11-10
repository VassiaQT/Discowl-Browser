use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Download {
    pub id: Uuid,
    pub url: String,
    pub file_path: PathBuf,
    pub total_bytes: Option<u64>,
    pub received_bytes: u64,
    pub state: DownloadState,
    pub started_at: SystemTime,
    pub completed_at: Option<SystemTime>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DownloadState {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

pub struct DownloadManager {
    downloads: Arc<RwLock<HashMap<Uuid, Download>>>,
}

impl DownloadManager {
    pub fn new() -> Self {
        Self { downloads: Arc::new(RwLock::new(HashMap::new())) }
    }

    pub fn start_download(&self, url: String, file_path: PathBuf) -> Uuid {
        let id = Uuid::new_v4();
        let download = Download {
            id,
            url,
            file_path,
            total_bytes: None,
            received_bytes: 0,
            state: DownloadState::Pending,
            started_at: SystemTime::now(),
            completed_at: None,
        };
        self.downloads.write().insert(id, download);
        id
    }

    pub fn update_progress(&self, id: Uuid, received: u64, total: Option<u64>) {
        if let Some(download) = self.downloads.write().get_mut(&id) {
            download.received_bytes = received;
            if let Some(t) = total {
                download.total_bytes = Some(t);
            }
            download.state = DownloadState::InProgress;
        }
    }

    pub fn complete(&self, id: Uuid) {
        if let Some(download) = self.downloads.write().get_mut(&id) {
            download.state = DownloadState::Completed;
            download.completed_at = Some(SystemTime::now());
        }
    }

    pub fn fail(&self, id: Uuid) {
        if let Some(download) = self.downloads.write().get_mut(&id) {
            download.state = DownloadState::Failed;
            download.completed_at = Some(SystemTime::now());
        }
    }

    pub fn cancel(&self, id: Uuid) {
        if let Some(download) = self.downloads.write().get_mut(&id) {
            download.state = DownloadState::Cancelled;
            download.completed_at = Some(SystemTime::now());
        }
    }

    pub fn get(&self, id: Uuid) -> Option<Download> {
        self.downloads.read().get(&id).cloned()
    }

    pub fn all(&self) -> Vec<Download> {
        let mut downloads: Vec<_> = self.downloads.read().values().cloned().collect();
        downloads.sort_by(|a, b| b.started_at.cmp(&a.started_at));
        downloads
    }

    pub fn clear_completed(&self) {
        self.downloads.write().retain(|_, d| !matches!(d.state, DownloadState::Completed | DownloadState::Failed | DownloadState::Cancelled));
    }
}

impl Default for DownloadManager {
    fn default() -> Self {
        Self::new()
    }
}