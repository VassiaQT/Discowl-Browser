use discowl_ipc::{CoreEvent, TabId, UiCommand};
use discowl_storage::Profile;
use parking_lot::RwLock;
use std::sync::Arc;
use std::sync::mpsc;

use crate::{BookmarkManager, DownloadManager, HistoryManager, SettingsManager, TabManager, navigation};

pub struct BrowserCore {
    pub tabs: Arc<TabManager>,
    pub history: Arc<HistoryManager>,
    pub bookmarks: Arc<BookmarkManager>,
    pub downloads: Arc<DownloadManager>,
    pub settings: Arc<SettingsManager>,
    pub profile: Arc<Profile>,
    event_tx: RwLock<Option<mpsc::Sender<CoreEvent>>>,
    sidebar_open: RwLock<bool>,
}

impl BrowserCore {
    pub fn new(profile: Profile) -> Result<Self, Box<dyn std::error::Error>> {
        let history_db = discowl_storage::HistoryDb::new(profile.history_db_path())?;
        let bookmarks_db = discowl_storage::BookmarksDb::new(profile.bookmarks_db_path())?;
        let settings_db = discowl_storage::SettingsDb::new(profile.settings_path())?;

        let history = Arc::new(HistoryManager::new());
        history.init(history_db);

        let bookmarks = Arc::new(BookmarkManager::new());
        bookmarks.init(bookmarks_db);

        let settings = Arc::new(SettingsManager::new());
        settings.init(settings_db);

        let tabs = Arc::new(TabManager::new());
        
        let homepage = settings.get().homepage.clone();
        let _first_tab = tabs.create_tab(homepage);

        Ok(Self {
            tabs,
            history,
            bookmarks,
            downloads: Arc::new(DownloadManager::new()),
            settings,
            profile: Arc::new(profile),
            event_tx: RwLock::new(None),
            sidebar_open: RwLock::new(false),
        })
    }

    pub fn set_event_sender(&self, tx: mpsc::Sender<CoreEvent>) {
        *self.event_tx.write() = Some(tx);
    }

    fn emit(&self, event: CoreEvent) {
        if let Some(tx) = self.event_tx.read().as_ref() {
            let _ = tx.send(event);
        }
    }

    pub fn handle_command(&self, cmd: UiCommand) {
        match cmd {
            UiCommand::NewTab { url } => {
                let url = url.map(|u| navigation::normalize_url(u)).unwrap_or_else(|| self.settings.get().homepage);
                self.tabs.create_tab(url);
                self.emit_tabs_changed();
            }
            UiCommand::CloseTab { id } => {
                self.tabs.close_tab(id);
                self.emit_tabs_changed();
            }
            UiCommand::SwitchTab { id } => {
                if self.tabs.switch_tab(id) {
                    self.emit_tabs_changed();
                }
            }
            UiCommand::Navigate { id, input } => {
                let url = navigation::normalize_url(input);
                if self.tabs.update_tab(id, |tab| {
                    tab.url = url.clone();
                    tab.loading = true;
                }) {
                    self.emit(CoreEvent::TabUpdated {
                        id,
                        title: "Loading...".to_string(),
                        url,
                        loading: true,
                        can_go_back: false,
                        can_go_forward: false,
                    });
                }
            }
            UiCommand::GoBack { id: _ } => {
                // Handled by engine
            }
            UiCommand::GoForward { id: _ } => {
                // Handled by engine
            }
            UiCommand::Reload { id } => {
                if self.tabs.update_tab(id, |tab| tab.loading = true) {
                    self.emit(CoreEvent::TabUpdated {
                        id,
                        title: "Loading...".to_string(),
                        url: self.tabs.get_tab(id).map(|t| t.url).unwrap_or_default(),
                        loading: true,
                        can_go_back: false,
                        can_go_forward: false,
                    });
                }
            }
            UiCommand::ToggleSidebar => {
                let mut open = self.sidebar_open.write();
                *open = !*open;
                self.emit(CoreEvent::SidebarToggled { open: *open });
            }
            UiCommand::OpenSettings => {
                let settings_url = "browser://settings".to_string();
                self.tabs.create_tab(settings_url);
                self.emit_tabs_changed();
            }
        }
    }

    pub fn on_tab_loaded(&self, id: TabId, title: String, url: String, can_go_back: bool, can_go_forward: bool) {
        self.tabs.update_tab(id, |tab| {
            tab.title = title.clone();
            tab.url = url.clone();
            tab.loading = false;
            tab.can_go_back = can_go_back;
            tab.can_go_forward = can_go_forward;
            if let Some(favicon) = navigation::get_favicon_url(&url) {
                tab.favicon = Some(favicon);
            }
        });
        
        self.history.add_visit(&url, &title);
        self.emit(CoreEvent::TabUpdated {
            id,
            title,
            url,
            loading: false,
            can_go_back,
            can_go_forward,
        });
    }

    pub fn on_tab_title_changed(&self, id: TabId, title: String) {
        self.tabs.update_tab(id, |tab| tab.title = title.clone());
        if let Some(tab) = self.tabs.get_tab(id) {
            self.emit(CoreEvent::TabUpdated {
                id,
                title,
                url: tab.url,
                loading: tab.loading,
                can_go_back: tab.can_go_back,
                can_go_forward: tab.can_go_forward,
            });
        }
    }

    pub fn on_tab_url_changed(&self, id: TabId, url: String) {
        self.tabs.update_tab(id, |tab| {
            tab.url = url.clone();
            if let Some(favicon) = navigation::get_favicon_url(&url) {
                tab.favicon = Some(favicon);
            }
        });
        if let Some(tab) = self.tabs.get_tab(id) {
            self.emit(CoreEvent::TabUpdated {
                id,
                title: tab.title,
                url,
                loading: tab.loading,
                can_go_back: tab.can_go_back,
                can_go_forward: tab.can_go_forward,
            });
        }
    }

    pub fn on_tab_loading_changed(&self, id: TabId, loading: bool) {
        self.tabs.update_tab(id, |tab| tab.loading = loading);
        if let Some(tab) = self.tabs.get_tab(id) {
            self.emit(CoreEvent::TabUpdated {
                id,
                title: tab.title,
                url: tab.url,
                loading,
                can_go_back: tab.can_go_back,
                can_go_forward: tab.can_go_forward,
            });
        }
    }

    pub fn add_bookmark(&self, url: &str, title: &str) {
        self.bookmarks.add(url, title);
    }

    pub fn remove_bookmark(&self, id: i64) {
        self.bookmarks.remove(id);
    }

    pub fn is_bookmarked(&self, url: &str) -> bool {
        self.bookmarks.is_bookmarked(url)
    }

    fn emit_tabs_changed(&self) {
        let tabs = self.tabs.all_tabs();
        let active = self.tabs.get_active().unwrap_or_else(TabId::new);
        self.emit(CoreEvent::TabsChanged { tabs, active });
    }
}