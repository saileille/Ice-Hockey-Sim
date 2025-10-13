use std::collections::HashMap;

// Functions that allow the user to interact with backend.
use ::time::Date;
use crate::{competition::{season::Season, Competition}, database::{COMPETITIONS, TODAY}, types::CompetitionId};

pub mod tests;

// Advance the time with one day.
#[tauri::command]
pub fn go_to_next_day() {
    let today = TODAY.lock().unwrap().clone();

    let comps = COMPETITIONS.lock().unwrap().clone();
    for comp in comps.values() {
        // Simulate all games that happen today.
        if comp.format.is_some() {
            let mut season = Season::fetch_from_db(&comp.id, comp.get_seasons_amount() - 1);
            season.simulate_day(&comp, &today);
        }
    }

    // DO EVERYTHING AGAIN


    // Check against days with no games.
    /*let mut games = match GAMES.lock().unwrap().get(&date_to_db_string(&today)) {
        Some(g) => g.values().cloned().collect(),
        _ => Vec::new()
    };

    for game in games.iter_mut() {
        game.play();
    }*/

    // Advance the day.
    *TODAY.lock().unwrap() = today.next_day().unwrap();
}