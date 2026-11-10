use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum UiCommand {
    NewTab { url: Option<String> },
    CloseTab { id: TabId },
    SwitchTab { id: TabId },
    Navigate { id: TabId, input: String },
    GoBack { id: TabId },
    GoForward { id: TabId },
    Reload { id: TabId },
    ToggleSidebar,
    OpenSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CoreEvent {
    TabsChanged { tabs: Vec<TabState>, active: TabId },
    TabUpdated { id: TabId, title: String, url: String, loading: bool, can_go_back: bool, can_go_forward: bool },
    SidebarToggled { open: bool },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TabId(pub Uuid);

impl TabId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for TabId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for TabId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabState {
    pub id: TabId,
    pub title: String,
    pub url: String,
    pub loading: bool,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub favicon: Option<String>,
}

impl TabState {
    pub fn new(id: TabId, url: String) -> Self {
        Self {
            id,
            title: "New Tab".to_string(),
            url,
            loading: false,
            can_go_back: false,
            can_go_forward: false,
            favicon: None,
        }
    }
}

impl UiCommand {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("UiCommand serialization failed")
    }
}

impl CoreEvent {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("CoreEvent serialization failed")
    }
}