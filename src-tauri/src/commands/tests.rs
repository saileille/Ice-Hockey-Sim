// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use crate::{
    database::COMPETITIONS,
    competition::Competition,
    match_event::Game,
    team::Team
};

#[tauri::command]
pub fn test_game() -> (String, String) {
    let mut home = Team::build_and_save("Home");
    let mut away = Team::build_and_save("Away");

    home.setup(0, 0);
    away.setup(0, 0);

    let mut game = Game::build(home.id, away.id, 1, "");
    game.play();

    let data = (game.get_name_and_score(), game.get_simple_boxscore());

    home.delete_from_db();
    away.delete_from_db();
    home.delete_players();
    away.delete_players();

    return data;
}

// Test a competition.
#[tauri::command]
pub fn test_comp() -> String {
    let comp = COMPETITIONS.lock().unwrap().get(&1).unwrap().clone();
    // comp.create_season(None);

    "".to_string()
}

/* #[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
} */