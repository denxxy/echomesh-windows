pub mod commands;
pub mod state;

use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::Manager;
use tracing_subscriber::EnvFilter;

use state::{get_app_data_path, AppState};

pub fn run() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .try_init();

    let storage_path = get_app_data_path();
    tracing::info!("EchoMesh storage path: {:?}", storage_path);

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .manage(AppState::new(storage_path))
        .setup(|app| {
            // Build Tray Icon Menu
            let open_i = MenuItem::with_id(app, "open", "Открыть EchoMesh", true, None::<&str>)?;
            let status_i = MenuItem::with_id(app, "status", "Статус: Отключен", false, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Выход", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open_i, &status_i, &quit_i])?;

            let tray_icon = app.default_window_icon().cloned();
            let mut tray_builder = TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("EchoMesh Client")
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                    "status" => {}
                    "quit" => {
                        let state = app.state::<AppState>();
                        let client_guard = state.client.blocking_read();
                        if let Some(ref client) = *client_guard {
                            let _ = client.disconnect();
                        }
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
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                });

            if let Some(icon) = tray_icon {
                tray_builder = tray_builder.icon(icon);
            }

            tray_builder.build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            // Minimize to system tray on window close button
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::connect_to_relay,
            commands::disconnect,
            commands::get_status,
            commands::get_contacts,
            commands::add_contact,
            commands::send_message,
            commands::get_history,
        ])
        .run(tauri::generate_context!())
        .expect("error while running echomesh tauri application");
}
