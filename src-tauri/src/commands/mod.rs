use std::cmp::Ordering;

use crate::{competition::{season::{team::TeamCompData, Season}, Competition}, database::{COMPETITIONS, SEASONS, TODAY}, team::Team, time::{date_to_db_string, db_string_to_date}, types::{CompetitionId, TeamId}};
use serde_json::json;

pub mod tests;

// Advance the time with one day.
#[tauri::command]
pub fn go_to_next_day() -> String {
    let today = TODAY.lock().unwrap().clone();

    let mut comps = COMPETITIONS.lock().unwrap().clone();
    for comp in comps.values_mut() {
        let mut season = Season::fetch_from_db(&comp.id, comp.get_seasons_amount() - 1);

        // Simulate all games that happen today.
        if comp.format.is_some() {
            season.simulate_day(&comp, &today);
        }

        // Create new seasons for competitions that are over.
        else if today > db_string_to_date(&season.end_date) {
            // Can only take the same teams for the next season as well, for now.
            let teams: Vec<TeamId> = season.teams.iter().map(|a | a.team_id).collect();
            comp.create_and_setup_seasons(&teams);
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

// Get name and ID of all competitions that are not part of another competition.
#[tauri::command]
pub fn get_all_full_competitions() -> Vec<(String, String)> {
    let mut comps: Vec<(String, String)> = COMPETITIONS.lock().unwrap().iter().filter_map(|(id, a)| {
        if a.parent_comp_id == 0 {
            Some((id.to_string(), a.name.clone()))
        }
        else {
            None
        }
    }).collect();

    // Let's make one default option...
    comps.push(("0".to_string(), "[Competitions]".to_string()));

    comps.sort_by(|a, b| {
        // Making sure that the non-selection is always on top.
        let ordering;
        if a.0 == "0" { ordering = Ordering::Less }
        else if b.0 == "0" { ordering = Ordering::Greater }
        else { ordering = a.1.cmp(&b.1) }
        ordering
    });

    return comps;
}

// Get name and ID of all competitions that are children of the given competition.
#[tauri::command]
pub fn get_child_competitions(id: CompetitionId) -> Vec<(String, String)> {
    let parent_comp = Competition::fetch_from_db(&id).unwrap();
    let mut child_comps: Vec<(String, String)> = parent_comp.child_comp_ids.iter().map(|a| (a.to_string(), Competition::fetch_from_db(a).unwrap().name)).collect();

    // Parent competition is the default option.
    child_comps.push(("0".to_string(), parent_comp.name));

    child_comps.sort_by(|a, b| {
        // Making sure that the non-selection is always on top.
        let ordering;
        if a.0 == "0" { ordering = Ordering::Less }
        else if b.0 == "0" { ordering = Ordering::Greater }
        else { ordering = a.1.cmp(&b.1) }
        ordering
    });

    return child_comps;
}

// Get all the info of a competition in a JSON string.
#[tauri::command]
pub fn get_comp_screen_info(id: CompetitionId) -> String {
    Competition::fetch_from_db(&id).unwrap().get_comp_screen_json().to_string()
}