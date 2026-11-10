use discowl_ipc::{TabId, TabState};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct TabManager {
    tabs: Arc<RwLock<HashMap<TabId, TabState>>>,
    active_tab: Arc<RwLock<Option<TabId>>>,
    order: Arc<RwLock<Vec<TabId>>>,
}

impl TabManager {
    pub fn new() -> Self {
        Self {
            tabs: Arc::new(RwLock::new(HashMap::new())),
            active_tab: Arc::new(RwLock::new(None)),
            order: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn create_tab(&self, url: String) -> TabId {
        let id = TabId::new();
        let state = TabState::new(id, url);
        
        self.tabs.write().insert(id, state);
        self.order.write().push(id);
        *self.active_tab.write() = Some(id);
        
        id
    }

    pub fn close_tab(&self, id: TabId) -> bool {
        let mut tabs = self.tabs.write();
        let mut order = self.order.write();
        
        if tabs.remove(&id).is_none() {
            return false;
        }
        
        order.retain(|&x| x != id);
        
        let mut active = self.active_tab.write();
        if *active == Some(id) {
            *active = order.last().copied();
        }
        
        true
    }

    pub fn switch_tab(&self, id: TabId) -> bool {
        if self.tabs.read().contains_key(&id) {
            *self.active_tab.write() = Some(id);
            true
        } else {
            false
        }
    }

    pub fn get_active(&self) -> Option<TabId> {
        *self.active_tab.read()
    }

    pub fn get_tab(&self, id: TabId) -> Option<TabState> {
        self.tabs.read().get(&id).cloned()
    }

    pub fn update_tab(&self, id: TabId, update: impl FnOnce(&mut TabState)) -> bool {
        let mut tabs = self.tabs.write();
        if let Some(tab) = tabs.get_mut(&id) {
            update(tab);
            true
        } else {
            false
        }
    }

    pub fn all_tabs(&self) -> Vec<TabState> {
        let tabs = self.tabs.read();
        let order = self.order.read();
        order.iter().filter_map(|id| tabs.get(id).cloned()).collect()
    }

    pub fn tab_count(&self) -> usize {
        self.tabs.read().len()
    }

    pub fn active_tab_state(&self) -> Option<TabState> {
        let active = *self.active_tab.read();
        let active = active?;
        self.tabs.read().get(&active).cloned()
    }
}

impl Default for TabManager {
    fn default() -> Self {
        Self::new()
    }
}