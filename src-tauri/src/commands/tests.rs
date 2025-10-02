// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use crate::match_event::Game;
use crate::team::Team;

#[tauri::command]
pub fn test_game() -> (String, String) {
    let mut home: Team = Team::create_and_save("Home");
    let mut away: Team = Team::create_and_save("Away");

    home.generate_roster(0, 0);
    away.generate_roster(0, 0);

    home.auto_build_lineup();
    away.auto_build_lineup();
    
    let mut game: Game = Game::new(home.id, away.id);
    game.play();
    
    let data: (String, String) = (game.get_name_and_score(), game.get_simple_boxscore());
    home.delete_from_db();
    away.delete_from_db();

    return data;
}

/* #[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
} */