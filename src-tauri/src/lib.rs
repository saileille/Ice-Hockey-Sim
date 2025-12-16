mod commands;
mod db;
mod logic;

use tauri::{Manager as _, async_runtime::block_on};

use crate::db::initialise;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let handle = app.handle();
            block_on(async move {
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
            commands::db::new_database,
            commands::db::load_database,
            commands::db::save_database,
            commands::db::to_main_menu,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}