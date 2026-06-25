mod app_state;
mod clipboard;
mod commands;
mod config;
mod db;
mod fonts;
mod locale;
mod logging;
mod paste;
mod tray;

use std::sync::atomic::Ordering;

use tauri::Manager;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
use winapi::um::winuser::GetAsyncKeyState;

use app_state::AppState;

pub fn run() {
    logging::init();

    let cfg = config::Config::load();
    let db_path = cfg.db.path.clone();
    let max_entries = cfg.max_entries;
    let hotkey = cfg.overlay_hotkey_plugin_format();

    let database = db::Database::open(&db_path, max_entries).expect("Failed to open database");

    let lang = cfg.locale.clone().unwrap_or_else(locale::detect_language);
    let locale_strings = locale::LocaleStrings::load(&lang);

    let state = AppState::new(cfg, database, locale_strings);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, shortcut, event| {
                    log::info!("shortcut: state={:?}, key={}", event.state, shortcut);
                    if event.state == ShortcutState::Pressed {
                        commands::toggle_overlay(app);
                    }
                })
                .build(),
        )
        .manage(state)
        .setup(move |app| {
            log::info!("setup started");
            setup_tray(app)?;
            setup_overlay_transparency(app)?;
            spawn_clipboard_monitor(app);
            spawn_escape_poller(app);
            register_hotkey(app, &hotkey)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_clipboard_entries,
            commands::ensure_clipboard_captured,
            commands::delete_entry,
            commands::save_edited_entry,
            commands::clear_entries,
            commands::paste_entry,
            commands::get_config,
            commands::save_config,
            commands::get_locale_strings,
            commands::get_system_fonts,
            commands::open_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let tray_locale = {
        let state = app.state::<AppState>();
        let guard = state.locale_strings.lock().map_err(|e| e.to_string())?;
        guard.strings.clone()
    };

    let tray_items = tray::setup(app.app_handle(), &tray_locale)?;
    {
        let state = app.state::<AppState>();
        *state.tray_items.lock().map_err(|e| e.to_string())? = Some(tray_items);
    }
    Ok(())
}

fn setup_overlay_transparency(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(overlay) = app.get_webview_window("overlay") {
        let _ = overlay.set_background_color(None);
    }
    Ok(())
}

fn register_hotkey(app: &tauri::App, hotkey: &str) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("registering hotkey: '{}'", hotkey);
    app.global_shortcut()
        .register(hotkey)
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn spawn_clipboard_monitor(app: &tauri::App) {
    let handle = app.app_handle().clone();
    let (tx, rx) = std::sync::mpsc::channel::<clipboard::ClipboardEvent>();
    clipboard::start_monitoring(tx);

    std::thread::spawn(move || {
        for event in rx {
            log::debug!("clipboard event received");

            if let Some(state) = handle.try_state::<AppState>() {
                if state.suppress_monitor.load(Ordering::SeqCst) {
                    log::debug!("skipping self-triggered clipboard event");
                    state.suppress_monitor.store(false, Ordering::SeqCst);
                    continue;
                }
                state.handle_clipboard_event(event);
            }
        }
    });
}

fn spawn_escape_poller(app: &tauri::App) {
    let handle = app.app_handle().clone();
    std::thread::spawn(move || {
        log::info!("escape polling thread started");
        let mut prev_down = false;
        loop {
            std::thread::sleep(std::time::Duration::from_millis(30));
            let down = unsafe { GetAsyncKeyState(0x1B) as u16 & 0x8000 != 0 };
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
