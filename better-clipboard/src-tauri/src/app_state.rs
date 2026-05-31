use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

use crate::{clipboard, config, db, locale, tray};

pub struct AppState {
    pub config: Mutex<config::Config>,
    pub database: Mutex<db::Database>,
    pub suppress_monitor: AtomicBool,
    pub overlay_visible: AtomicBool,
    pub locale_strings: Mutex<locale::LocaleStrings>,
    pub tray_items: Mutex<Option<tray::TrayItems<tauri::Wry>>>,
}

impl AppState {
    pub fn new(cfg: config::Config, database: db::Database, locale_strings: locale::LocaleStrings) -> Self {
        Self {
            config: Mutex::new(cfg),
            database: Mutex::new(database),
            suppress_monitor: AtomicBool::new(false),
            overlay_visible: AtomicBool::new(false),
            locale_strings: Mutex::new(locale_strings),
            tray_items: Mutex::new(None),
        }
    }

    pub fn handle_clipboard_event(&self, event: clipboard::ClipboardEvent) {
        let (entry_type, text_content, file_data) = match event {
            clipboard::ClipboardEvent::Text(text) => {
                ("text".to_string(), Some(text), None)
            }
            clipboard::ClipboardEvent::Image(data) => {
                ("image".to_string(), None, Some(data))
            }
        };

        let content_hash = Self::hash_text(text_content.as_deref().unwrap_or(""));

        if let Ok(db_lock) = self.database.lock() {
            let display_order = db_lock.next_display_order().unwrap_or(0);
            let entry = db::ClipboardEntry {
                id: 0,
                entry_type,
                content_hash,
                text_content,
                file_path: None,
                thumbnail_path: None,
                file_size: file_data.map(|d| d.len() as i64),
                source_app: None,
                created_at: String::new(),
                is_pinned: false,
                display_order,
            };
            let _ = db_lock.insert_or_update(&entry);
        }
    }

    pub fn hash_text(text: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(text.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}
