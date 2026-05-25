use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    pub file_path: PathBuf,
    pub icon_pending: String,
    pub icon_clear: String,
    pub popup_width: u32,
    pub popup_height: u32,
}

impl Default for Config {
    fn default() -> Self {
        let file_path = dirs::home_dir()
            .map(|h| h.join("todo.md"))
            .unwrap_or_else(|| PathBuf::from("todo.md"));
        Self {
            file_path,
            icon_pending: "checkbox-symbolic".to_string(),
            icon_clear: "checkbox-checked-symbolic".to_string(),
            popup_width: 380,
            popup_height: 620,
        }
    }
}

impl Config {
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .map(|c| c.join("cosmic-applet-todo").join("config.toml"))
            .unwrap_or_else(|| PathBuf::from("cosmic-applet-todo.toml"))
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        let Ok(raw) = std::fs::read_to_string(&path) else {
            return Self::default();
        };
        match toml::from_str::<Self>(&raw) {
            Ok(cfg) => cfg,
            Err(e) => {
                log::warn!("Failed to parse {}, using defaults: {}", path.display(), e);
                Self::default()
            }
        }
    }
}
