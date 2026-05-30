mod clipboard;
mod config;
mod db;
mod fonts;
mod locale;
mod logging;
mod paste;
mod tray;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

pub struct AppState {
    pub config: Mutex<config::Config>,
    pub database: Mutex<db::Database>,
    pub suppress_monitor: AtomicBool,
    pub overlay_visible: AtomicBool,
    pub locale_strings: Mutex<locale::LocaleStrings>,
    pub tray_items: Mutex<Option<tray::TrayItems<tauri::Wry>>>,
}

#[tauri::command]
fn get_clipboard_entries(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<db::ClipboardEntry>, String> {
    let db = state.database.lock().map_err(|e| e.to_string())?;
    db.get_recent(10).map_err(|e| e.to_string())
}

#[tauri::command]
fn ensure_clipboard_captured(state: tauri::State<'_, AppState>) -> Result<(), String> {
    use sha2::{Digest, Sha256};

    let text = match clipboard::read_current_text() {
        Some(t) if !t.is_empty() => t,
        _ => return Ok(()),
    };

    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    let content_hash = format!("{:x}", hasher.finalize());

    let db = state.database.lock().map_err(|e| e.to_string())?;
    let existing = db
        .get_id_by_hash(&content_hash, "text")
        .map_err(|e| e.to_string())?;

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
fn delete_entry(state: tauri::State<'_, AppState>, id: i64) -> Result<(), String> {
    let db = state.database.lock().map_err(|e| e.to_string())?;
    db.delete_entry(id).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_edited_entry(
    state: tauri::State<'_, AppState>,
    id: i64,
    text: String,
) -> Result<(), String> {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    let content_hash = format!("{:x}", hasher.finalize());
    let db = state.database.lock().map_err(|e| e.to_string())?;
    db.update_entry_text(id, &text, &content_hash)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_config(state: tauri::State<'_, AppState>) -> Result<config::Config, String> {
    state
        .config
        .lock()
        .map_err(|e| e.to_string())
        .map(|g| g.clone())
}

#[tauri::command]
fn save_config(
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
    {
        let mut guard = state.config.lock().map_err(|e| e.to_string())?;
        *guard = config;
    }
    {
        let mut locale_guard = state.locale_strings.lock().map_err(|e| e.to_string())?;
        *locale_guard = new_locale.clone();
    }
    {
        let tray_guard = state.tray_items.lock().map_err(|e| e.to_string())?;
        if let Some(ref items) = *tray_guard {
            tray::update_tray_menu(items, &new_locale.strings);
        }
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
fn get_locale_strings(
    state: tauri::State<'_, AppState>,
) -> Result<std::collections::HashMap<String, String>, String> {
    let guard = state.locale_strings.lock().map_err(|e| e.to_string())?;
    Ok(guard.strings.clone())
}

#[tauri::command]
fn get_system_fonts() -> Vec<String> {
    fonts::get_system_families()
}

#[tauri::command]
fn open_settings(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(overlay) = app.get_webview_window("overlay") {
        let _ = overlay.hide();
    }
    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.show();
        let _ = window.set_focus();
    } else {
        log::warn!("settings window not found via command, creating...");
        if let Ok(window) = tauri::WebviewWindowBuilder::new(
            &app,
            "settings",
            tauri::WebviewUrl::App("settings.html".into()),
        )
        .title("Better Clipboard - 設定")
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
fn paste_entry(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    index: usize,
) -> Result<(), String> {
    log::info!("paste_entry called: index={}", index);

    let db = state.database.lock().map_err(|e| e.to_string())?;
    let entries = db.get_recent(10).map_err(|e| e.to_string())?;
    let entry = entries.get(index).ok_or("Invalid index")?;
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

        std::thread::sleep(std::time::Duration::from_millis(50));

        log::info!("pasting text: {} chars", text.len());
        paste::paste_text(&text)?;
        log::info!("paste done");
    }
    Ok(())
}

#[tauri::command]
fn clear_entries(
    state: tauri::State<'_, AppState>,
    mode: String,
    days: Option<i64>,
) -> Result<(), String> {
    log::info!("clear_entries: mode={}, days={:?}", mode, days);
    let db = state.database.lock().map_err(|e| e.to_string())?;
    let result = match mode.as_str() {
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
        _ => Err("Invalid mode".to_string()),
    };
    drop(db);

    if mode == "all" && result.is_ok() {
        log::info!("clear_all: emptying system clipboard");
        #[cfg(target_os = "windows")]
        unsafe {
            use winapi::shared::minwindef::FALSE;
            use winapi::um::winuser::{CloseClipboard, EmptyClipboard, OpenClipboard};
            let opened = OpenClipboard(std::ptr::null_mut());
            if opened != FALSE {
                EmptyClipboard();
                CloseClipboard();
            }
        }
    }

    log::info!("clear_entries result: {:?}", result);
    result
}

fn toggle_overlay(app: &tauri::AppHandle) {
    log::info!("toggle_overlay");
    if let Some(window) = app.get_webview_window("overlay") {
        if let Some(state) = app.try_state::<AppState>() {
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
    } else {
        log::warn!("overlay window not found");
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    logging::init();

    let cfg = config::Config::load();
    let db_path = cfg.db.path.clone();
    let max_entries = cfg.max_entries;
    let hotkey = cfg.overlay_hotkey_plugin_format();

    let database = db::Database::open(&db_path, max_entries).expect("Failed to open database");

    let lang = cfg.locale.clone().unwrap_or_else(locale::detect_language);
    let locale_strings = locale::LocaleStrings::load(&lang);

    let state = AppState {
        config: Mutex::new(cfg),
        database: Mutex::new(database),
        suppress_monitor: AtomicBool::new(false),
        overlay_visible: AtomicBool::new(false),
        locale_strings: Mutex::new(locale_strings),
        tray_items: Mutex::new(None),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, shortcut, event| {
                    log::info!(
                        "SHORTCUT HANDLER CALLED: state={:?}, shortcut={}",
                        event.state,
                        shortcut
                    );
                    if event.state == ShortcutState::Pressed {
                        toggle_overlay(app);
                    }
                })
                .build(),
        )
        .manage(state)
        .setup(move |app| {
            log::info!("setup started");

            let tray_locale = {
                let state = app.state::<AppState>();
                let guard = state.locale_strings.lock().map_err(|e| e.to_string())?;
                guard.strings.clone()
            };

            let tray_items =
                tray::setup(app.app_handle(), &tray_locale).map_err(|e| e.to_string())?;
            {
                let state = app.state::<AppState>();
                let mut guard = state.tray_items.lock().map_err(|e| e.to_string())?;
                *guard = Some(tray_items);
            }
            log::info!("registering hotkey: '{}'", hotkey);
            let result = app.global_shortcut().register(hotkey.as_str());
            log::info!("registration result: {:?}", result);
            result.map_err(|e| format!("Failed to register hotkey: {}", e))?;

            // Set overlay webview background to transparent (fixes white corners on Windows)
            if let Some(overlay) = app.get_webview_window("overlay") {
                let _ = overlay.set_background_color(None);
            }

            let handle = app.app_handle().clone();
            let (tx, rx) = std::sync::mpsc::channel::<clipboard::ClipboardEvent>();
            clipboard::start_monitoring(tx);

            std::thread::spawn(move || {
                use sha2::{Digest, Sha256};
                for event in rx {
                    log::debug!("clipboard event received");

                    if let Some(state) = handle.try_state::<AppState>() {
                        if state.suppress_monitor.load(Ordering::SeqCst) {
                            log::debug!("skipping self-triggered clipboard event");
                            state.suppress_monitor.store(false, Ordering::SeqCst);
                            continue;
                        }
                    }

                    let (entry_type, text_content, file_data) = match event {
                        clipboard::ClipboardEvent::Text(text) => {
                            ("text".to_string(), Some(text), None)
                        }
                        clipboard::ClipboardEvent::Image(data) => {
                            ("image".to_string(), None, Some(data))
                        }
                    };

                    let text_for_hash = text_content.as_deref().unwrap_or("");
                    let mut hasher = Sha256::new();
                    hasher.update(text_for_hash.as_bytes());
                    let content_hash = format!("{:x}", hasher.finalize());

                    if let Some(db) = handle.try_state::<AppState>() {
                        if let Ok(db_lock) = db.database.lock() {
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
                }
            });

            // Escape key polling (transparent windows on Windows don't receive keyboard events)
            {
                let handle = app.app_handle().clone();
                std::thread::spawn(move || {
                    log::info!("escape polling thread started");
                    let mut prev_down = false;
                    loop {
                        std::thread::sleep(std::time::Duration::from_millis(30));
                        let down = unsafe {
                            winapi::um::winuser::GetAsyncKeyState(0x1B) as u16 & 0x8000 != 0
                        };
                        if down && !prev_down {
                            if let Some(state) = handle.try_state::<AppState>() {
                                if state.overlay_visible.load(Ordering::SeqCst) {
                                    log::info!("escape detected, hiding overlay");
                                    if let Some(window) = handle.get_webview_window("overlay") {
                                        let _ = window.hide();
                                        state.overlay_visible.store(false, Ordering::SeqCst);
                                    }
                                }
                            }
                        }
                        prev_down = down;
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_clipboard_entries,
            ensure_clipboard_captured,
            delete_entry,
            save_edited_entry,
            clear_entries,
            paste_entry,
            get_config,
            save_config,
            get_locale_strings,
            get_system_fonts,
            open_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
