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
mod tests;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    database::initialise();

    // Test stuffs...
    #[cfg(dev)] {
        // tests::simulate_to_day("2026-05-01");
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
            commands::continue_game::go_to_next_day,
            commands::continue_game::get_date_string,
            commands::get_all_full_competitions,
            commands::get_child_competitions,
            commands::get_comp_screen_info,
            commands::get_team_screen_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}