use tauri::{AppHandle, Manager as _};

use crate::{db::AppData, logic::CompetitionId, packages::{CompetitionSelect, editor::Competition}};

// Get name and ID of all competitions that are not part of another competition.
#[tauri::command]
pub async fn comp_select_package(handle: AppHandle) -> Vec<CompetitionSelect> {
    let data = handle.state::<AppData>();
    let comps = CompetitionSelect::all(&data).await;

    return comps;
}

// Get data for a competition's editor view based on the ID.
#[tauri::command]
pub async fn comp_editor_package(handle: AppHandle, id: CompetitionId) -> Competition {
    let data = handle.state::<AppData>();
    if id == 0 {
        // Used for creating new competitions.
        return Competition::create_default(&data.db_info.pool).await;
    }
    else {
        return Competition::from_id(id, &data.db_info.pool).await;
    }
}