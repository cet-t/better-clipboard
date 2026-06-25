use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::time::Duration;

use tauri::{Emitter, Manager};

use crate::app_state::AppState;
use crate::{clipboard, config, db, fonts, locale, paste, tray};

macro_rules! lock {
    ($mutex:expr) => {
        $mutex.lock().map_err(|e| e.to_string())
    };
}

#[tauri::command]
pub fn get_clipboard_entries(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<db::ClipboardEntry>, String> {
    let db = lock!(state.database)?;
    db.get_recent(10).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn ensure_clipboard_captured(
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let text = match clipboard::read_current_text() {
        Some(t) if !t.is_empty() => t,
        _ => return Ok(()),
    };

    let content_hash = AppState::hash_text(&text);
    let db = lock!(state.database)?;
    let existing = db.get_id_by_hash(&content_hash, "text").map_err(|e| e.to_string())?;

    if existing.is_none() {
        let display_order = db.next_display_order().map_err(|e| e.to_string())?;
        let entry = db::ClipboardEntry {
            id: 0,
            entry_type: "text".to_string(),
            content_hash,
            text_content: Some(text),
            file_path: None,
            thumbnail_path: None,
            file_size: None,
            source_app: None,
            created_at: String::new(),
            is_pinned: false,
            display_order,
        };
        db.insert_or_update(&entry).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn delete_entry(
    state: tauri::State<'_, AppState>,
    id: i64,
) -> Result<(), String> {
    let db = lock!(state.database)?;
    db.delete_entry(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_edited_entry(
    state: tauri::State<'_, AppState>,
    id: i64,
    text: String,
) -> Result<(), String> {
    let content_hash = AppState::hash_text(&text);

    let sync_clipboard = {
        let db = lock!(state.database)?;
        let old_text = db.get_entry_text(id).map_err(|e| e.to_string())?;
        db.update_entry_text(id, &text, &content_hash)
            .map_err(|e| e.to_string())?;

        match (old_text, clipboard::read_current_text()) {
            (Some(old), Some(current)) => old == current,
            _ => false,
        }
    };

    if sync_clipboard {
        state.suppress_monitor.store(true, Ordering::SeqCst);
        paste::set_clipboard_text(&text).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub fn get_config(
    state: tauri::State<'_, AppState>,
) -> Result<config::Config, String> {
    let guard = lock!(state.config)?;
    Ok(guard.clone())
}

#[tauri::command]
pub fn save_config(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    config: config::Config,
) -> Result<(), String> {
    let lang = config
        .locale
        .clone()
        .unwrap_or_else(locale::detect_language);
    let new_locale = locale::LocaleStrings::load(&lang);
    config.save();

    *lock!(state.config)? = config;
    *lock!(state.locale_strings)? = new_locale.clone();

    let tray_guard = lock!(state.tray_items)?;
    if let Some(ref items) = *tray_guard {
        tray::update_tray_menu(items, &new_locale.strings);
    }

    if let Some(w) = app.get_webview_window("overlay") {
        let _ = w.set_title(&new_locale.get("window_title_overlay"));
    }
    if let Some(w) = app.get_webview_window("settings") {
        let _ = w.set_title(&new_locale.get("window_title_settings"));
    }
    Ok(())
}

#[tauri::command]
pub fn get_locale_strings(
    state: tauri::State<'_, AppState>,
) -> Result<HashMap<String, String>, String> {
    let guard = lock!(state.locale_strings)?;
    Ok(guard.strings.clone())
}

#[tauri::command]
pub fn get_system_fonts() -> Vec<String> {
    fonts::get_system_families()
}

#[tauri::command]
pub fn open_settings(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(overlay) = app.get_webview_window("overlay") {
        let _ = overlay.hide();
    }
    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.show();
        let _ = window.set_focus();
    } else {
        log::warn!("settings window not found, creating it now");
        if let Ok(window) = tauri::WebviewWindowBuilder::new(
            &app,
            "settings",
            tauri::WebviewUrl::App("settings.html".into()),
        )
        .title("Better Clipboard - Settings")
        .inner_size(520.0, 600.0)
        .resizable(false)
        .center()
        .build()
        {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
    Ok(())
}

#[tauri::command]
pub fn paste_entry(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    index: usize,
) -> Result<(), String> {
    log::info!("paste_entry called: index={}", index);

    let db = lock!(state.database)?;
    let entries = db.get_recent(10).map_err(|e| e.to_string())?;
    let entry = entries.get(index).ok_or("invalid entry index")?;
    log::info!(
        "entry found: type={}, text_len={}",
        entry.entry_type,
        entry.text_content.as_ref().map_or(0, |s| s.len())
    );

    if let Some(text) = &entry.text_content {
        let text = text.clone();
        drop(db);
        state.suppress_monitor.store(true, Ordering::SeqCst);

        if let Some(window) = app.get_webview_window("overlay") {
            let _ = window.hide();
        }

        std::thread::sleep(Duration::from_millis(50));

        log::info!("pasting text: {} chars", text.len());
        paste::paste_text(&text).map_err(|e| e.to_string())?;
        log::info!("paste done");
    }
    Ok(())
}

#[tauri::command]
pub fn clear_entries(
    state: tauri::State<'_, AppState>,
    mode: String,
    days: Option<i64>,
) -> Result<(), String> {
    log::info!("clear_entries: mode={}, days={:?}", mode, days);

    let result = {
        let db = lock!(state.database)?;
        match mode.as_str() {
            "display" => db.clear_display_only().map_err(|e| e.to_string()),
            "all" => {
                let count = db.count_entries().map_err(|e| e.to_string())?;
                log::info!("clear_all: entries before delete: {}", count);
                let r = db.clear_all().map_err(|e| e.to_string());
                if r.is_ok() {
                    let after = db.count_entries().map_err(|e| e.to_string())?;
                    log::info!("clear_all: entries after delete: {}", after);
                }
                r
            }
            "older" => db
                .clear_older_than(days.unwrap_or(30))
                .map_err(|e| e.to_string()),
            _ => Err("invalid clear mode".to_string()),
        }
    };

    if mode == "all" && result.is_ok() {
        log::info!("clear_all: emptying system clipboard");
        clipboard::empty();
    }

    log::info!("clear_entries result: {:?}", result);
    result
}

pub fn toggle_overlay(app: &tauri::AppHandle) {
    log::info!("toggle_overlay");
    let Some(window) = app.get_webview_window("overlay") else {
        log::warn!("overlay window not found");
        return;
    };
    let Some(state) = app.try_state::<AppState>() else {
        return;
    };

    if window.is_visible().unwrap_or(false) {
        log::info!("hiding overlay");
        let _ = window.hide();
        state.overlay_visible.store(false, Ordering::SeqCst);
    } else {
        if let Some(settings) = app.get_webview_window("settings") {
            let _ = settings.hide();
        }
        log::info!("showing overlay");
        let _ = window.show();
        let _ = window.set_focus();
        let _ = window.center();
        let _ = window.eval("document.getElementById('overlay')?.focus()");
        let _ = window.emit("refresh-entries", ());
        state.overlay_visible.store(true, Ordering::SeqCst);
    }
}
