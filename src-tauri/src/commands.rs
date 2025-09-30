use std::collections::HashMap;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use crate::game::Game;
use crate::team;

#[tauri::command]
pub fn test_game() -> [String; 2] {
    team::create_team("Home".to_string());
    team::create_team("Away".to_string());

    let mut game: Game = Game::new(1, 2);
    game.simulate();

    return [game.get_name_and_score(), game.get_simple_boxscore()];
}

/* #[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
} */