// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Emitter, Manager};
use tauri_plugin_updater::UpdaterExt;

#[derive(Clone, serde::Serialize)]
struct Payload {
  args: Vec<String>,
  cwd: String,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(debug_assertions)]
    let devtools = tauri_plugin_devtools::init(); // initialize the plugin as early as possible

    let builder = tauri::Builder::default();
    let context = tauri::generate_context!();

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
            println!("{}, {argv:?}, {cwd}", app.package_info().name);
            let window = app.get_webview_window("main").unwrap();
            window.show().unwrap();
            app.emit("single-instance", Payload { args: argv, cwd })
                .unwrap();
        }))
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                window.hide().unwrap();
                api.prevent_close();
            }
            _ => {}
        })
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let _response = handle.updater().unwrap().check().await;
            });

            let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
            let update = MenuItemBuilder::with_id("update", "Check for updates").build(app)?;
            let hide = MenuItemBuilder::with_id("hide", "Hide").build(app)?;
            let show = MenuItemBuilder::with_id("show", "Show").build(app)?;
            let relaunch = MenuItemBuilder::with_id("relaunch", "Relaunch").build(app)?;
            let seperator = PredefinedMenuItem::separator(app)?;
            let menu = MenuBuilder::new(app)
                .items(&[
                    &hide, &show, &seperator, &update, &relaunch, &seperator, &quit,
                ])
                .build()?;
            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("Akademia")
                .icon(app.default_window_icon().unwrap().clone())
                .menu_on_left_click(false)
                .on_menu_event(move |app, event| match event.id().as_ref() {
                    "quit" => {
                        app.exit(1);
                    }
                    "hide" => {
                        let window = app.get_webview_window("main").unwrap();
                        window.hide().unwrap();
                    }
                    "show" => {
                        let window = app.get_webview_window("main").unwrap();
                        window.show().unwrap();
                    }
                    "relaunch" => {
                        tauri::process::restart(&app.env());
                    }
                    "update" => {
                        let handle = app.app_handle().clone();
                        tauri::async_runtime::spawn(async move {
                            let _response = handle.updater().unwrap().check().await;
                        });
                    }
                    _ => (),
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
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;
            Ok(())
        })
        .run(context)
        .expect("error while running tauri application");
}
