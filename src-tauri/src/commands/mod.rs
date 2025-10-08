// Functions that allow the user to interact with backend.
use ::time::Date;
use crate::{
    database::{GAMES, TODAY},
    time::date_to_db_string,
    match_event::Game
};

pub mod tests;

// Advance the time with one day.
#[tauri::command]
pub fn go_to_next_day() {
    // Simulate all games that happen today.
    let today: Date = TODAY.lock().unwrap().clone();

    // Check against days with no games.
    let mut games: Vec<Game> = match GAMES.lock().unwrap().get(&date_to_db_string(&today)) {
        Some(g) => g.values().cloned().collect(),
        _ => Vec::new()
    };

    for game in games.iter_mut() {
        game.play();
    }

    // Advance the day.
    *TODAY.lock().unwrap() = today.next_day().unwrap();
}