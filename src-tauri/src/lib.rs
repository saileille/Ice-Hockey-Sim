mod commands;
mod competition;
mod country;
mod custom_types;
mod database;
mod event;
mod match_event;
mod io;
mod person;
mod team;
mod terminal_tests;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    database::initialise();

    // terminal testing...
    #[cfg(dev)] {
        // terminal_tests::test_match_generator();
    }

    tauri::Builder::default()
        .setup(|app| {
            #[cfg(debug_assertions)] {
                let window: tauri::WebviewWindow = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![commands::tests::test_game])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}