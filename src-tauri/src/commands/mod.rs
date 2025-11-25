pub mod continue_game;

use std::cmp::Ordering;

use serde_json::json;
use tauri::Manager as TauriManager;

use crate::{app_data::AppData, competition::{self, Competition}, database, person::{Contract, ContractRole, Person, manager::Manager, player::Player}, team::Team, time::date_to_string, types::{CompetitionId, PersonId, TeamId}};


// Get name and ID of all competitions that are not part of another competition.
#[tauri::command]
pub async fn comp_select_package(handle: tauri::AppHandle) -> Vec<(CompetitionId, String)> {
    let db = &handle.state::<AppData>().db;
    let mut comps = Competition::fetch_parent_id_and_name(db).await;

    // Let's make one default option...
    comps.push((0, "[Competitions]".to_string()));

    comps.sort_by(|a, b| {
        // Making sure that the non-selection is always on top.
        let ordering;
        if a.0 == 0 { ordering = Ordering::Less }
        else if b.0 == 0 { ordering = Ordering::Greater }
        else { ordering = a.1.cmp(&b.1) }
        ordering
    });

    return comps;
}

// Get name and ID of teams that are part of a competition.
#[tauri::command]
pub async fn team_select_package(handle: tauri::AppHandle, id: CompetitionId) -> Vec<(TeamId, String)> {
    let db = &handle.state::<AppData>().db;

    let mut teams = Competition::current_season_team_select_data_by_id(db, id).await;

    // The default option that does nothing.
    teams.push((0, "[No Team]".to_string()));

    teams.sort_by(|a, b| {
        // Making sure that the non-selection is always on top.
        let ordering;
        if a.0 == 0 { ordering = Ordering::Less }
        else if b.0 == 0 { ordering = Ordering::Greater }
        else { ordering = a.1.cmp(&b.1) }
        ordering
    });

    return teams;
}

// Get all the info for a competition screen in a JSON string.
#[tauri::command]
pub async fn comp_screen_package(handle: tauri::AppHandle, id: CompetitionId) -> serde_json::Value {
    let db = &handle.state::<AppData>().db;
    let comp = Competition::fetch_from_db(db, id).await;

    if comp.comp_type == competition::Type::Tournament {
        return comp.get_tournament_comp_screen_package(db).await;
    }
    else {
        return comp.get_comp_screen_package(db).await;
    }
}

// Get all info for a team screen in a JSON string.
#[tauri::command]
pub async fn team_screen_package(handle: tauri::AppHandle, id: TeamId) -> serde_json::Value {
    let db = &handle.state::<AppData>().db;
    Team::fetch_from_db(db, id).await.team_screen_package(db).await
}

// Get info for a player screen in a JSON string.
#[tauri::command]
pub async fn player_package(handle: tauri::AppHandle, id: PersonId) -> serde_json::Value {
    let db = &handle.state::<AppData>().db;
    Player::fetch_from_db(db, id).await.unwrap().package(db).await
}

// Create a human manager in the game.
#[tauri::command]
pub async fn create_human_manager(handle: tauri::AppHandle, id: TeamId) {
    let db = &handle.state::<AppData>().db;
    let (total_weight, country_weights) = Person::country_weights(db).await;
    let mut human = Manager::build_and_save_random(db, &country_weights, total_weight, true).await;
    human.is_human = true;

    // human.person.forename = "Human".to_string();
    // human.person.surname = "Manager".to_string();

    // This would mean the manager is unemployed.
    if id == 0 {
        println!("Started as unemployed.");
        return;
    }
    let team = Team::fetch_from_db(db, id).await;
    Contract::build_from_years(db, human.person.id, &team, 100, ContractRole::Manager, true).await;

    println!("Took control of {}.", team.full_name);
}

// Get information about the human.
#[tauri::command]
pub async fn human_package(handle: tauri::AppHandle) -> serde_json::Value {
    let db = &handle.state::<AppData>().db;
    let human = Manager::humans(db).await;
    return human[0].package(db).await;
}

// Get all free agents.
#[tauri::command]
pub async fn free_agents_package(handle: tauri::AppHandle) -> Vec<serde_json::Value> {
    let db = &handle.state::<AppData>().db;
    Player::free_agents_package(db).await
}

// Offer a contract to a player.
#[tauri::command]
pub async fn offer_contract(handle: tauri::AppHandle, player_id: PersonId, team_id: TeamId, years: i32) {
    let db = &handle.state::<AppData>().db;

    let mut team = Team::fetch_from_db(db, team_id).await;
    team.send_contract_offer(db, player_id, years, ContractRole::Player).await;

    // This is for players to evaluate this team's attractiveness.
    team.evaluate_player_needs(db).await;
}

// Get relevant information for the top bar.
#[tauri::command]
pub async fn top_bar_package(handle: tauri::AppHandle) -> serde_json::Value {
    let db = &handle.state::<AppData>().db;
    json!({
        "date": date_to_string(database::get_today(db).await),
        "human": human_package(handle).await,
    })
}