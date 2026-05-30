use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hotkeys {
    #[serde(default = "default_overlay")]
    pub overlay: String,
    #[serde(default = "default_select_keys")]
    pub select_keys: String,
    #[serde(default = "default_edit_key")]
    pub edit_key: String,
    #[serde(default = "default_delete_key")]
    pub delete_key: String,
    #[serde(default = "default_page_up")]
    pub page_up: String,
    #[serde(default = "default_page_down")]
    pub page_down: String,
}

fn default_overlay() -> String {
    "alt+c".to_string()
}

fn default_select_keys() -> String {
    "asdfjkl;".to_string()
}

fn default_edit_key() -> String {
    "e".to_string()
}

fn default_delete_key() -> String {
    "q".to_string()
}

fn default_page_up() -> String {
    "w".to_string()
}

fn default_page_down() -> String {
    "r".to_string()
}

impl Default for Hotkeys {
    fn default() -> Self {
        Self {
            overlay: default_overlay(),
            select_keys: default_select_keys(),
            edit_key: default_edit_key(),
            delete_key: default_delete_key(),
            page_up: default_page_up(),
            page_down: default_page_down(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum PersistenceMode {
    #[default]
    #[serde(rename = "session")]
    Session,
    #[serde(rename = "db")]
    Db,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbConfig {
    #[serde(default = "default_db_path")]
    pub path: PathBuf,
}

fn default_db_path() -> PathBuf {
    let base = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("BetterClipboard").join("content.db")
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            path: default_db_path(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub hotkeys: Hotkeys,
    #[serde(default)]
    pub persistence: PersistenceMode,
    #[serde(default)]
    pub db: DbConfig,
    #[serde(default = "default_max_entries")]
    pub max_entries: usize,
    #[serde(default)]
    pub font_family: Option<String>,
    #[serde(default)]
    pub locale: Option<String>,
}

fn default_max_entries() -> usize {
    100
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hotkeys: Hotkeys::default(),
            persistence: PersistenceMode::default(),
            db: DbConfig::default(),
            max_entries: default_max_entries(),
            font_family: None,
            locale: None,
        }
    }
}

impl Config {
    pub fn overlay_hotkey_plugin_format(&self) -> String {
        self.hotkeys
            .overlay
            .split(&['+', ' '][..])
            .filter(|s| !s.is_empty())
            .map(|s| {
                let s = s.trim();
                if s.len() == 1 {
                    s.to_uppercase()
                } else {
                    let mut chars = s.chars();
                    chars.next().unwrap().to_uppercase().to_string() + chars.as_str()
                }
            })
            .collect::<Vec<_>>()
            .join("+")
    }

    pub fn path() -> PathBuf {
        let base = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
        base.join("BetterClipboard").join("config.toml")
    }

    pub fn load() -> Self {
        let path = Self::path();
        if !path.exists() {
            let config = Config::default();
            config.save();
            return config;
        }
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        toml::from_str(&content).unwrap_or_default()
    }

    pub fn save(&self) {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let content = toml::to_string_pretty(self).unwrap_or_default();
        let _ = std::fs::write(&path, content);
    }
}
