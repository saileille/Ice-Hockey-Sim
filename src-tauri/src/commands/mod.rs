pub mod continue_game;

use std::cmp::Ordering;

use crate::{competition::Competition, database::COMPETITIONS, team::Team, types::{CompetitionId, TeamId}};


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
    let parent_comp = Competition::fetch_from_db(&id);
    let mut child_comps: Vec<(String, String)> = parent_comp.child_comp_ids.iter().map(|a| (a.to_string(), Competition::fetch_from_db(a).name)).collect();

    // Parent competition is the default option.
    child_comps.push(("0".to_string(), parent_comp.name));

    // Sort according to the ID, meaning the earlier stage should always be first.
    // Could be sorted with start dates too, but that would require extracting comps from the db.
    child_comps.sort_by(|a, b| a.0.cmp(&b.0));

    return child_comps;
}

// Get all the info for a competition screen in a JSON string.
#[tauri::command]
pub fn get_comp_screen_info(id: CompetitionId) -> String {
    let comp = Competition::fetch_from_db(&id);

    if comp.is_tournament_tree {
        return comp.get_tournament_comp_screen_json().to_string();
    }
    else {
        return comp.get_comp_screen_json().to_string();
    }
}

// Get all info for a team screen in a JSON string.
#[tauri::command]
pub fn get_team_screen_info(id: TeamId) -> String {
    Team::fetch_from_db(&id).get_team_screen_json().to_string()
}