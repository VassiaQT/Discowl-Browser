use directories::ProjectDirs;
use std::path::PathBuf;

pub struct Profile {
    pub data_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub config_dir: PathBuf,
}

impl Profile {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let proj = ProjectDirs::from("org", "discowl", "discowl-browser")
            .ok_or("Could not determine project directories")?;

        let data_dir = proj.data_dir().to_path_buf();
        let cache_dir = proj.cache_dir().to_path_buf();
        let config_dir = proj.config_dir().to_path_buf();

        std::fs::create_dir_all(&data_dir)?;
        std::fs::create_dir_all(&cache_dir)?;
        std::fs::create_dir_all(&config_dir)?;

        Ok(Self {
            data_dir,
            cache_dir,
            config_dir,
        })
    }

    pub fn history_db_path(&self) -> PathBuf {
        self.data_dir.join("history.db")
    }

    pub fn bookmarks_db_path(&self) -> PathBuf {
        self.data_dir.join("bookmarks.db")
    }

    pub fn settings_path(&self) -> PathBuf {
        self.config_dir.join("settings.json")
    }

    pub fn downloads_dir(&self) -> PathBuf {
        let dir = self.data_dir.join("downloads");
        std::fs::create_dir_all(&dir).ok();
        dir
    }

    pub fn servo_storage_path(&self) -> PathBuf {
        let dir = self.data_dir.join("servo");
        std::fs::create_dir_all(&dir).ok();
        dir
    }
}

impl Default for Profile {
    fn default() -> Self {
        Self::new().expect("Failed to create profile")
    }
}