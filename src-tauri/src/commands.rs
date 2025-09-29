// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use crate::game;

#[tauri::command]
pub fn test_game() -> [String; 2] {
    let mut game = game::build_game();
    game.simulate();

    return [game.get_name_and_score(), game.get_simple_boxscore()];
}

/* #[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
} */