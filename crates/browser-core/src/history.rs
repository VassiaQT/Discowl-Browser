use discowl_storage::{HistoryDb, HistoryEntry};
use parking_lot::RwLock;
use std::sync::Arc;

pub struct HistoryManager {
    db: Arc<RwLock<Option<HistoryDb>>>,
}

impl HistoryManager {
    pub fn new() -> Self {
        Self { db: Arc::new(RwLock::new(None)) }
    }

    pub fn init(&self, db: HistoryDb) {
        *self.db.write() = Some(db);
    }

    pub fn add_visit(&self, url: &str, title: &str) {
        if let Some(db) = self.db.read().as_ref() {
            let _ = db.add_visit(url, title);
        }
    }

    pub fn get_recent(&self, limit: usize) -> Vec<HistoryEntry> {
        self.db.read().as_ref().and_then(|db| db.get_recent(limit).ok()).unwrap_or_default()
    }

    pub fn search(&self, query: &str, limit: usize) -> Vec<HistoryEntry> {
        self.db.read().as_ref().and_then(|db| db.search(query, limit).ok()).unwrap_or_default()
    }
}

impl Default for HistoryManager {
    fn default() -> Self {
        Self::new()
    }
}