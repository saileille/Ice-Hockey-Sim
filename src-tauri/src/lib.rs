mod commands;
mod database;
mod game;
mod io;
mod person;
mod team;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![commands::test_game])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
