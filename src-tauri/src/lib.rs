mod commands;
mod competition;
mod country;
mod types;
mod database;
mod event;
mod match_event;
mod io;
mod person;
mod team;
mod time;
mod terminal_tests;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    database::initialise();

    // terminal testing...
    #[cfg(dev)] {
        terminal_tests::test_comp_generation();
    }

    tauri::Builder::default()
        .setup(|app| {
            #[cfg(debug_assertions)] {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![commands::tests::test_game])
        .invoke_handler(tauri::generate_handler![commands::tests::test_comp])
        .invoke_handler(tauri::generate_handler![commands::go_to_next_day])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}