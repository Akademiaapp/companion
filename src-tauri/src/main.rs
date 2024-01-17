// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{
    CustomMenuItem, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem, Manager, plugin::TauriPlugin,
};


#[derive(Clone, serde::Serialize)]
struct Payload {
  args: Vec<String>,
  cwd: String,
}

fn main() {
    // Create the tray menu
    let quit: CustomMenuItem = CustomMenuItem::new("quit".to_string(), "Quit");
    let update: CustomMenuItem = CustomMenuItem::new("update".to_string(), "Check for updates");
    let hide: CustomMenuItem = CustomMenuItem::new("hide".to_string(), "Hide");
    let show: CustomMenuItem = CustomMenuItem::new("show".to_string(), "Show");
    let relaunch: CustomMenuItem = CustomMenuItem::new("relaunch".to_string(), "Relaunch");
    let tray_menu: SystemTrayMenu = SystemTrayMenu::new()
        .add_item(hide)
        .add_item(show)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(update)
        .add_item(relaunch)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);

    // Create the tray
    let tray: SystemTray = SystemTray::new().with_menu(tray_menu);

    #[cfg(debug_assertions)] // only enable instrumentation in development builds
    let devtools: TauriPlugin<tauri::Wry> = devtools::init();

    let builder = tauri::Builder::default();

    #[cfg(debug_assertions)]
    let builder = builder.plugin(devtools);
    // Run the tauri application
    builder
        .on_page_load(|window, payload| {
            if payload.url().to_string() == "about:blank" && std::env::consts::OS == "linux" {
                let _result = window.eval(&format!("window.location.replace('tauri://localhost');"));
            }
        })
        .plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
            let window: tauri::Window = app.get_window("main").unwrap();
            window.show().unwrap();
            app.emit_all("single-instance", Payload { args: argv, cwd }).unwrap();
        }))
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .on_window_event(|event| match event.event() {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                event.window().hide().unwrap();
                api.prevent_close();
            }
            _ => {}
        })
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::LeftClick {
                position: _,
                size: _,
                ..
            } => {
                let window: tauri::Window = app.get_window("main").unwrap();
                window.show().unwrap();
                window.set_focus().unwrap();
            }
            SystemTrayEvent::DoubleClick {
                position: _,
                size: _,
                ..
            } => {
                let window: tauri::Window = app.get_window("main").unwrap();
                window.show().unwrap();
                window.set_focus().unwrap();
            }
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "quit" => {
                    app.exit(1);
                }
                "hide" => {
                    let window: tauri::Window = app.get_window("main").unwrap();
                    window.hide().unwrap();
                }
                "show" => {
                    let window: tauri::Window = app.get_window("main").unwrap();
                    window.show().unwrap();
                }
                "relaunch" => {
                    tauri::api::process::restart(&app.env());
                }
                "update" => {
                    app.updater();
                }
                _ => {}
            },
            _ => {}
        })
        .system_tray(tray)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
