mod commands;
mod competition;
mod country;
mod database;
mod event;
mod match_event;
mod io;
mod person;
mod team;

use competition::stage::{Stage, rules};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    database::initialise();


    // console testing...
    #[cfg(dev)] {
        let stage: Stage = Stage::build(
            "blbl",
            Vec::from([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]),
            rules::RoundRobin::build(0, 12),
        );

        for _ in 0..10 {
            stage.matches_for_round_robin();
        }
    }
    // console testing over, do as you like.


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
