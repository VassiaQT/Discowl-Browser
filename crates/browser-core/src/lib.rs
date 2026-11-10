pub mod bookmarks;
pub mod command;
pub mod downloads;
pub mod history;
pub mod navigation;
pub mod settings;
pub mod tabs;

pub use bookmarks::BookmarkManager;
pub use command::BrowserCore;
pub use downloads::{Download, DownloadManager, DownloadState};
pub use history::HistoryManager;
pub use settings::{AppearanceSettings, PrivacySettings, Settings, SettingsManager, Theme};
pub use tabs::TabManager;