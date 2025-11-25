// Commands and helper functions that have to do with continuing the game.

use tauri::Manager as TauriManager;

use crate::{app_data::AppData, competition::Competition, database, person::{Contract, Person, manager::Manager, player::Player}, team::Team, types::Db};


// Advance the time with one day.
#[tauri::command]
pub async fn go_to_next_day(handle: tauri::AppHandle) {
    let db = &handle.state::<AppData>().db;

    Contract::delete_expired(db).await;
    handle_players(db).await;
    handle_managers_and_teams(db).await;

    // Games are simulated here - this must be the last one!
    handle_comps(db).await;

    database::next_day(db).await;
}

// Do the daily tasks of competitions.
async fn handle_comps(db: &Db) {
    let today = database::get_today(db).await;

    let comps_with_games = Competition::fetch_comps_with_games(db).await;
    let parents = Competition::fetch_parents(db).await;
    for comp in comps_with_games {
        let season = comp.current_season(db).await;

        // Simulate all games that happen today.
        season.simulate_day(db).await;
    }

    for comp in parents {
        let season = comp.current_season(db).await;

        // Create new seasons for parent competitions whose seasons are over.
        if today > season.end_date {
            // Cannot change teams between seasons, for now.
            let teams = season.team_ids(db).await;
            comp.create_and_setup_seasons(db, &teams).await;
        }
    }
}

// Do the daily tasks of managers (and teams, they are connected).
async fn handle_managers_and_teams(db: &Db) {
    let managers = Manager::fetch_active(db).await;
    let (total_weight, country_weights) = Person::country_weights(db).await;

    for manager in managers.into_iter() {
        let o_contract= manager.person.contract(db).await;

        // TODO: have the manager look for a job or something.
        if o_contract.is_none() {
            continue;
        }
        let contract = o_contract.unwrap();
        let mut team = Team::fetch_from_db(db, contract.team_id).await;

        // Initial AI evaluation here.
        // Done for human managers as well so the players can evaluate the contract offers they receive.
        team.evaluate_player_needs(db).await;

        // Do not do anything on behalf of the human.
        if manager.is_human {
            team.return_actions_to_full(db).await;
            team.season_end_checker(db, &country_weights, total_weight).await;
            continue;
        }

        while team.actions_remaining > 0 {
            let contract_offered = team.offer_contract(db).await;
            if !contract_offered {
                break;
            }
            else {
                team.evaluate_player_needs(db).await;
            }
        }

        team.return_actions_to_full(db).await;
        team.season_end_checker(db, &country_weights, total_weight).await;
    }

    // TODO: teams without managers should do tasks specific to them.
    /*
    let mut teams = TEAMS.lock().unwrap().clone();
    for team in teams.values_mut() {

    }
    */
}

// Do the daily tasks of players.
async fn handle_players(db: &Db) {
    let players = Player::fetch_active(db).await;

    for mut player in players.into_iter() {
        let signs_contract = player.person.decide_to_sign(db).await;
        if signs_contract {
            player.choose_contract(db).await;
        }

        // Player thinks about retiring
        if player.retires(db).await {
            player.person.retire(db).await;
            continue;
        }

        // Training after choosing the contract sounds most fair,
        // as then the player will choose their contract based on the most recent
        // information available to the managers, both human and AI.
        player.daily_training(db).await;
    }
}