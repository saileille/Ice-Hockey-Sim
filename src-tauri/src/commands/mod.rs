pub mod continue_game;

use std::cmp::Ordering;

use serde_json::json;
use tauri::Manager as TauriManager;

use crate::{db::get_today, logic::{app_data::AppData, competition::Competition, person::{contract::{Contract, ContractRole}, manager::Manager, player::Player}, team::Team, time::date_to_string, types::{CompetitionId, PersonId, TeamId}}, packages::screens::{competition, player_search}};

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
pub async fn comp_screen_package(handle: tauri::AppHandle, id: CompetitionId) -> competition::Package {
    let db = &handle.state::<AppData>().db;
    let today = get_today(db).await;
    competition::Package::build(db, today, id).await
}

// Get all info for a team screen in a JSON string.
#[tauri::command]
pub async fn team_screen_package(handle: tauri::AppHandle, id: TeamId) -> serde_json::Value {
    let db = &handle.state::<AppData>().db;
    let today = get_today(db).await;
    Team::fetch_from_db(db, id).await.team_screen_package(db, today).await
}

// Get info for a player screen in a JSON string.
#[tauri::command]
pub async fn player_package(handle: tauri::AppHandle, id: PersonId) -> serde_json::Value {
    let db = &handle.state::<AppData>().db;
    let today = get_today(db).await;
    Player::fetch_from_db(db, id).await.unwrap().package(db, today).await
}

// Create a human manager in the game.
#[tauri::command]
pub async fn create_human_manager(handle: tauri::AppHandle, id: TeamId) {
    let data = handle.state::<AppData>();
    let today = get_today(&data.db).await;
    let mut human = Manager::build_and_save_random(&data, today, true).await;
    human.is_human = true;

    // human.person.forename = "Human".to_string();
    // human.person.surname = "Manager".to_string();

    // This would mean the manager is unemployed.
    if id == 0 {
        println!("Started as unemployed.");
        return;
    }
    let team = Team::fetch_from_db(&data.db, id).await;
    Contract::build_from_years(&data.db, today, human.person.id, &team, 100, ContractRole::Manager, true).await;

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
pub async fn free_agents_package(handle: tauri::AppHandle) -> Vec<player_search::Package> {
    let now = std::time::Instant::now();

    let db = &handle.state::<AppData>().db;
    let today = get_today(db).await;
    let package = player_search::Package::free_agents(db, today).await;

    println!("Free agents fetched in {:.2?}", now.elapsed());
    return package;
}

// Offer a contract to a player.
#[tauri::command]
pub async fn offer_contract(handle: tauri::AppHandle, player_id: PersonId, team_id: TeamId, years: i32) {
    let db = &handle.state::<AppData>().db;
    let today = get_today(db).await;

    let mut team = Team::fetch_from_db(db, team_id).await;
    team.send_contract_offer(db, today, player_id, years, ContractRole::Player).await;

    // This is for players to evaluate this team's attractiveness.
    team.evaluate_player_needs(db).await;
}

// Get relevant information for the top bar.
#[tauri::command]
pub async fn top_bar_package(handle: tauri::AppHandle) -> serde_json::Value {
    let db = &handle.state::<AppData>().db;
    json!({
        "date": date_to_string(get_today(db).await),
        "human": human_package(handle).await,
    })
}