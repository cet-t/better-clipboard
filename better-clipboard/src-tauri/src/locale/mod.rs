use std::collections::HashMap;
use std::path::PathBuf;

use serde::Deserialize;
use winapi::um::winnls::GetUserDefaultUILanguage;

#[derive(Debug, Clone, Deserialize)]
pub struct LocaleStrings {
    #[serde(flatten)]
    pub strings: HashMap<String, String>,
}

impl LocaleStrings {
    pub fn load(lang: &str) -> Self {
        if let Some(s) = Self::from_file(lang) {
            log::info!("loaded locale from file: {}", lang);
            return s;
        }
        if let Some(s) = Self::from_embedded(lang) {
            log::info!("loaded locale from embedded: {}", lang);
            return s;
        }
        log::warn!("locale '{}' not found, using fallback", lang);
        Self::fallback()
    }

    fn from_file(lang: &str) -> Option<Self> {
        let path = locale_dir().join(format!("{}.json", lang));
        let content = std::fs::read_to_string(&path).ok()?;
        serde_json::from_str(&content).ok()
    }

    fn from_embedded(lang: &str) -> Option<Self> {
        let json = match lang {
            "ja" => Some(include_str!("../../locales/ja.json")),
            _ => None,
        };
        json.and_then(|s| serde_json::from_str(s).ok())
    }

    pub fn fallback() -> Self {
        let content = include_str!("../../locales/en.json");
        serde_json::from_str(content).expect("built-in en.json is valid")
    }

    pub fn get(&self, key: &str) -> String {
        self.strings
            .get(key)
            .cloned()
            .unwrap_or_else(|| key.to_string())
    }

    pub fn get_with(&self, key: &str, params: &[(&str, &str)]) -> String {
        let mut s = self.get(key);
        for (k, v) in params {
            s = s.replace(&format!("{{{}}}", k), v);
        }
        s
    }
}

fn locale_dir() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            return dir.join("locales");
        }
    }
    PathBuf::from("locales")
}

pub fn detect_language() -> String {
    unsafe {
        let lang_id = GetUserDefaultUILanguage();
        let primary = lang_id & 0x3FF;
        match primary {
            0x11 => "ja".to_string(),
            _ => "en".to_string(),
        }
    }
}
