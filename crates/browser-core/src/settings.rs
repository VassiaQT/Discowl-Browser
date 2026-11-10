use discowl_storage::SettingsDb;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub homepage: String,
    pub search_engine: String,
    pub theme: Theme,
    pub privacy: PrivacySettings,
    pub appearance: AppearanceSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Light,
    Dark,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacySettings {
    pub block_third_party_cookies: bool,
    pub do_not_track: bool,
    pub clear_on_exit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceSettings {
    pub show_bookmarks_bar: bool,
    pub compact_mode: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            homepage: "https://duckduckgo.com".to_string(),
            search_engine: "duckduckgo".to_string(),
            theme: Theme::System,
            privacy: PrivacySettings {
                block_third_party_cookies: true,
                do_not_track: true,
                clear_on_exit: false,
            },
            appearance: AppearanceSettings {
                show_bookmarks_bar: true,
                compact_mode: false,
            },
        }
    }
}

pub struct SettingsManager {
    db: Arc<RwLock<Option<SettingsDb>>>,
    settings: Arc<RwLock<Settings>>,
}

impl SettingsManager {
    pub fn new() -> Self {
        Self {
            db: Arc::new(RwLock::new(None)),
            settings: Arc::new(RwLock::new(Settings::default())),
        }
    }

    pub fn init(&self, db: SettingsDb) {
        *self.db.write() = Some(db);
        self.load();
    }

    pub fn load(&self) {
        if let Some(db) = self.db.read().as_ref() {
            if let Ok(Some(json)) = db.get("settings") {
                if let Ok(settings) = serde_json::from_str(&json) {
                    *self.settings.write() = settings;
                }
            }
        }
    }

    pub fn save(&self) {
        if let Some(db) = self.db.read().as_ref() {
            if let Ok(json) = serde_json::to_string(&*self.settings.read()) {
                let _ = db.set("settings", &json);
            }
        }
    }

    pub fn get(&self) -> Settings {
        self.settings.read().clone()
    }

    pub fn update(&self, f: impl FnOnce(&mut Settings)) {
        f(&mut self.settings.write());
        self.save();
    }
}

impl Default for SettingsManager {
    fn default() -> Self {
        Self::new()
    }
}