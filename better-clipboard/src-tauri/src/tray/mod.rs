use tauri::{
    AppHandle, Emitter, Runtime, WebviewUrl, WebviewWindowBuilder,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    menu::{Menu, MenuItem},
    Manager,
};

pub fn setup<R: Runtime>(app: &AppHandle<R>) -> Result<(), Box<dyn std::error::Error>> {
    let settings = MenuItem::with_id(app, "settings", "設定", true, None::<&str>)?;
    let restart = MenuItem::with_id(app, "restart", "再起動", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "終了", true, None::<&str>)?;

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
                    log::info!("found settings window, showing");
                    let _ = window.show();
                    let _ = window.set_focus();
                } else {
                    log::warn!("settings window not found, creating it now");
                    if let Ok(window) = WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("settings.html".into()))
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

    Ok(())
}
