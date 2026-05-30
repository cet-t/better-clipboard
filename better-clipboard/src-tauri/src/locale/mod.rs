use serde::Deserialize;
use std::collections::HashMap;

const LOCALE_JSON: &[(&str, &str)] = &[
    ("en", include_str!("../../locales/en.json")),
    ("ja", include_str!("../../locales/ja.json")),
];

#[derive(Debug, Clone, Deserialize)]
pub struct LocaleStrings {
    #[serde(flatten)]
    pub strings: HashMap<String, String>,
}

impl LocaleStrings {
    pub fn load(lang: &str) -> Self {
        let content = LOCALE_JSON
            .iter()
            .find(|(key, _)| *key == lang)
            .map(|(_, json)| *json)
            .unwrap_or(LOCALE_JSON[0].1);

        match serde_json::from_str::<Self>(content) {
            Ok(parsed) => {
                log::info!("loaded locale: {}", lang);
                parsed
            }
            Err(e) => {
                log::error!("failed to parse locale '{}': {}", lang, e);
                Self::fallback()
            }
        }
    }

    pub fn fallback() -> Self {
        let content = LOCALE_JSON[0].1;
        serde_json::from_str(content).expect("built-in en.json is valid")
    }

    pub fn get(&self, key: &str) -> String {
        self.strings.get(key).cloned().unwrap_or_else(|| key.to_string())
    }

    pub fn get_with(&self, key: &str, params: &[(&str, &str)]) -> String {
        let mut s = self.get(key);
        for (k, v) in params {
            s = s.replace(&format!("{{{}}}", k), v);
        }
        s
    }
}

pub fn detect_language() -> String {
    #[cfg(target_os = "windows")]
    {
        use winapi::um::winnls::GetUserDefaultUILanguage;
        let lang_id = unsafe { GetUserDefaultUILanguage() };
        let primary = lang_id & 0x3FF;
        match primary {
            0x11 => "ja".to_string(),
            _ => "en".to_string(),
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let locale = std::env::var("LANG").unwrap_or_default();
        if locale.starts_with("ja") { "ja".to_string() } else { "en".to_string() }
    }
}
