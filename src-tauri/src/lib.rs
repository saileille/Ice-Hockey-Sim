mod commands;
mod db;
mod logic;
mod packages;

use tauri::Manager;

use crate::db::initialise;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {

    // Test stuffs...
    #[cfg(dev)] {
        // tests::simulate_to_day("2026-05-01");
    }

    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle();
            tauri::async_runtime::block_on(async move {
                let data = initialise(handle).await;
                handle.manage(data);
            });

            #[cfg(debug_assertions)] {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::continue_game::go_to_next_day,
            commands::top_bar_package,
            commands::comp_select_package,
            commands::team_select_package,
            commands::comp_screen_package,
            commands::team_screen_package,
            commands::player_package,
            commands::create_human_manager,
            commands::human_package,
            commands::free_agents_package,
            commands::offer_contract,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}