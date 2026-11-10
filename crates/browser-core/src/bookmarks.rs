use discowl_storage::{BookmarksDb, Bookmark};
use parking_lot::RwLock;
use std::sync::Arc;

pub struct BookmarkManager {
    db: Arc<RwLock<Option<BookmarksDb>>>,
}

impl BookmarkManager {
    pub fn new() -> Self {
        Self { db: Arc::new(RwLock::new(None)) }
    }

    pub fn init(&self, db: BookmarksDb) {
        *self.db.write() = Some(db);
    }

    pub fn add(&self, url: &str, title: &str) -> Option<i64> {
        self.db.read().as_ref().and_then(|db| db.add(url, title, None).ok())
    }

    pub fn remove(&self, id: i64) -> bool {
        self.db.read().as_ref().and_then(|db| db.remove(id).ok()).is_some()
    }

    pub fn get_all(&self) -> Vec<Bookmark> {
        self.db.read().as_ref().and_then(|db| db.get_all().ok()).unwrap_or_default()
    }

    pub fn is_bookmarked(&self, url: &str) -> bool {
        self.get_all().iter().any(|b| b.url == url)
    }
}

impl Default for BookmarkManager {
    fn default() -> Self {
        Self::new()
    }
}