use tauri::{AppHandle, Manager as _};
use tauri_plugin_dialog::DialogExt as _;

use crate::db::AppData;

#[tauri::command]
pub async fn new_database(handle: AppHandle) {
    let data = handle.state::<AppData>();

    // Ask if the user wants to save unsaved changes.
    data.db_info.remind_save_dialog(&handle).await;

    // Pick a path for the new database file.
    let file_path = match handle.dialog()
    .file()
    .set_directory(data.directories.db.as_path())
    .set_file_name("db.db")
    .add_filter("Database Files", &["db"])
    .blocking_save_file() {
        Some(p) => p,
        None => return
    };

    // Switch the save file to the new database.
    *data.db_info.path.lock().unwrap() = file_path.to_string();

    // TODO: Some way to reset the database.
    data.db_info.reset().await;

    // Save the data to the database.
    data.db_info.save_to_file().await;
}

#[tauri::command]
pub async fn load_database(handle: AppHandle) {
    let data = handle.state::<AppData>();

    // Ask if the user wants to save unsaved changes.
    data.db_info.remind_save_dialog(&handle).await;

    // Pick the new database file.
    let file_path = match handle.dialog()
    .file()
    .set_directory(data.directories.db.as_path())
    .add_filter("Database Files", &["db"])
    .blocking_pick_file() {
        Some(p) => p,
        None => return
    };

    *data.db_info.path.lock().unwrap() = file_path.to_string();
    data.db_info.load_from_file().await;
}

#[tauri::command]
pub async fn save_database(handle: AppHandle) {
    let data = handle.state::<AppData>();
    data.db_info.save_to_file().await;
}

#[tauri::command]
pub async fn to_main_menu(handle: AppHandle) {
    let data = handle.state::<AppData>();

    // Ask if the user wants to save unsaved changes.
    data.db_info.remind_save_dialog(&handle).await;

    if !*data.db_info.in_sync.lock().unwrap() {
        data.db_info.load_from_file().await;
    }
}