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

use crate::database::TODAY;
use crate::time::db_string_to_date;
use crate::commands::go_to_next_day;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    database::initialise();

    // Test stuffs...
    #[cfg(dev)] {
        /* let simulate_to = "2026-05-01";    // Simulate to this date.
        loop {
            let today = TODAY.lock().unwrap().clone();
            if today > db_string_to_date(simulate_to) {
                break;
            }

            go_to_next_day();
        } */
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
        .invoke_handler(tauri::generate_handler![
            commands::go_to_next_day,
            commands::get_date_string,
            commands::get_all_full_competitions,
            commands::get_child_competitions,
            commands::get_comp_screen_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}