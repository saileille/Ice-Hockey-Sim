pub mod continue_game;

use std::cmp::Ordering;

use serde_json::json;
use time::Date;

use crate::{competition::{self, Competition}, database::{COMPETITIONS, TODAY}, person::{Contract, manager::Manager, player::Player}, team::Team, time::date_to_db_string, types::{CompetitionId, PlayerId, TeamId}};


// Get name and ID of all competitions that are not part of another competition.
#[tauri::command]
pub fn get_comp_select_package() -> Vec<(String, String)> {
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

// Get name and ID of teams that are part of a competition.
#[tauri::command]
pub fn get_team_select_package(id: CompetitionId) -> Vec<(String, String)> {
    let teams = Competition::fetch_from_db(&id).get_teams();
    let mut select_options: Vec<(String, String)> = teams.iter().map(|a| (a.id.to_string(), a.name.clone())).collect();

    // The default option that does nothing.
    select_options.push(("0".to_string(), "[No Team]".to_string()));

    select_options.sort_by(|a, b| {
        // Making sure that the non-selection is always on top.
        let ordering;
        if a.0 == "0" { ordering = Ordering::Less }
        else if b.0 == "0" { ordering = Ordering::Greater }
        else { ordering = a.1.cmp(&b.1) }
        ordering
    });

    return select_options;
}

// Get all the info for a competition screen in a JSON string.
#[tauri::command]
pub fn get_comp_screen_package(id: CompetitionId) -> serde_json::Value {
    let comp = Competition::fetch_from_db(&id);

    if comp.competition_type == competition::Type::Tournament {
        return comp.get_tournament_comp_screen_package();
    }
    else {
        return comp.get_comp_screen_package();
    }
}

// Get all info for a team screen in a JSON string.
#[tauri::command]
pub fn get_team_screen_package(id: TeamId) -> serde_json::Value {
    Team::fetch_from_db(&id).get_team_screen_package(&TODAY.lock().unwrap().clone())
}

// Get info for a player screen in a JSON string.
#[tauri::command]
pub fn get_player_package(id: PlayerId) -> serde_json::Value {
    Player::fetch_from_db(&id).unwrap().get_package(&TODAY.lock().unwrap().clone())
}

// Create a human manager in the game.
#[tauri::command]
pub fn create_human_manager(id: TeamId) {
    let mut human = Manager::build_and_save_random(&TODAY.lock().unwrap().clone(), &mut rand::rng());
    human.is_human = true;

    human.person.forename = "Human".to_string();
    human.person.surname = "Manager".to_string();

    // This would mean the manager is unemployed.
    if id == 0 {
        human.save();
        println!("Started as unemployed.");
        return;
    }

    let mut team = Team::fetch_from_db(&id);
    human.person.contract = Some(Contract::build(&date_to_db_string(&TODAY.lock().unwrap()), &date_to_db_string(&Date::MAX), id));

    match team.get_manager() {
        Some(mut manager) => {
            manager.person.contract = None;
            manager.save();
        },
        None => {}
    };

    team.manager_id = human.id;
    human.save();
    team.save();

    println!("Took control of {}.", team.name);
}

// Get information about the human.
#[tauri::command]
pub fn get_human_package() -> serde_json::Value {
    let human = Manager::get_human().unwrap();
    return human.get_package();
}

// Get all free agents.
#[tauri::command]
pub fn get_free_agents_package() -> serde_json::Value {
    Player::get_all_free_agents_package(&TODAY.lock().unwrap().clone())
}

// Offer a contract to a player.
#[tauri::command]
pub fn offer_contract(player_id: PlayerId, team_id: TeamId, years: i32) {
    let today = TODAY.lock().unwrap().clone();
    let mut team = Team::fetch_from_db(&team_id);
    let mut player = Player::fetch_from_db(&player_id).unwrap();
    let contract = Contract::build_from_years(&team, &today, years);

    team.offer_contract_to_player(&mut player, contract);

    // This is for players to evaluate this team's attractiveness.
    team.evaluate_player_needs();
    team.save();
}

// Get relevant information for the top bar.
#[tauri::command]
pub fn get_top_bar_package() -> serde_json::Value {
    json!({
        "date": date_to_db_string(&TODAY.lock().unwrap()),
        "human": get_human_package(),
    })
}