use std::collections::HashMap;

// Functions that allow the user to interact with backend.
use ::time::Date;
use crate::{competition::{season::Season, Competition}, database::{COMPETITIONS, TODAY}, time::{date_to_db_string, db_string_to_date}, types::CompetitionId};

pub mod tests;

// Advance the time with one day.
#[tauri::command]
pub fn go_to_next_day() -> String {
    let today = TODAY.lock().unwrap().clone();

    let comps = COMPETITIONS.lock().unwrap().clone();
    for comp in comps.values() {
        // Simulate all games that happen today.
        if comp.format.is_some() {
            let mut season = Season::fetch_from_db(&comp.id, comp.get_seasons_amount() - 1);
            season.simulate_day(&comp, &today);
        }
    }

    *TODAY.lock().unwrap() = today.next_day().unwrap();
    return date_to_db_string(&TODAY.lock().unwrap());
}

// Get the current date as a string.
#[tauri::command]
pub fn get_date_string() -> String {
    date_to_db_string(&TODAY.lock().unwrap())
}

// Get all competitions that are not part of another competition.
#[tauri::command]
pub fn get_all_full_competitions() {
    // let comps = HashMap::new();
}