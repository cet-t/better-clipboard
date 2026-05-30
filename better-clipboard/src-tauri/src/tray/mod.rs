use std::collections::HashMap;

use anyhow::Result;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, Runtime, WebviewUrl, WebviewWindowBuilder,
};

pub struct TrayItems<R: Runtime> {
    pub settings: MenuItem<R>,
    pub restart: MenuItem<R>,
    pub quit: MenuItem<R>,
}

pub fn setup<R: Runtime>(app: &AppHandle<R>, locale: &HashMap<String, String>) -> Result<TrayItems<R>> {
    let s = |k: &str| locale.get(k).cloned().unwrap_or_default();

    let settings = MenuItem::with_id(app, "settings", s("tray_settings"), true, None::<&str>)?;
    let restart = MenuItem::with_id(app, "restart", s("tray_restart"), true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", s("tray_quit"), true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&settings, &restart, &quit])?;

    TrayIconBuilder::new()
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "settings" => {
                log::info!("Settings clicked");
                if let Some(overlay) = app.get_webview_window("overlay") {
                    let _ = overlay.hide();
                }
                if let Some(window) = app.get_webview_window("settings") {
                    let _ = window.show();
                    let _ = window.set_focus();
                } else {
                    log::warn!("settings window not found, creating it now");
                    if let Ok(window) = WebviewWindowBuilder::new(
                        app,
                        "settings",
                        WebviewUrl::App("settings.html".into()),
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
            }
            "restart" => {
                log::info!("Restart clicked");
                app.restart();
            }
            "quit" => {
                log::info!("Quit clicked");
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                log::info!("Tray icon clicked");
                let _ = tray.app_handle().emit("toggle-overlay", ());
            }
        })
        .build(app)?;

    Ok(TrayItems { settings, restart, quit })
}

pub fn update_tray_menu<R: Runtime>(items: &TrayItems<R>, locale: &HashMap<String, String>) {
    let _ = items
        .settings
        .set_text(locale.get("tray_settings").map(|s| s.as_str()).unwrap_or("Settings"));
    let _ = items
        .restart
        .set_text(locale.get("tray_restart").map(|s| s.as_str()).unwrap_or("Restart"));
    let _ = items
        .quit
        .set_text(locale.get("tray_quit").map(|s| s.as_str()).unwrap_or("Quit"));
}
