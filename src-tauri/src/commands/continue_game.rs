// Commands and helper functions that have to do with continuing the game.

use std::collections::{HashMap, HashSet};

use rand::rngs::ThreadRng;
use time::Date;

use crate::{competition::season::Season, country::Country, database::{COMPETITIONS, COUNTRIES, MANAGERS, PLAYERS, TODAY}, person::player::Player, team::Team, time::db_string_to_date, types::{CountryId, TeamId}};


// Advance the time with one day.
#[tauri::command]
pub fn go_to_next_day() {
    let mut rng = rand::rng();
    let today = TODAY.lock().unwrap().clone();
    let countries = COUNTRIES.lock().unwrap().clone();

    handle_players(&today, &mut rng);
    handle_managers_and_teams(&countries, &today, &mut rng);

    // Games are simulated here - this must be the last one!
    handle_comps(&today, &mut rng);

    *TODAY.lock().unwrap() = today.next_day().unwrap();
}

// Do the daily tasks of competitions.
fn handle_comps(today: &Date, rng: &mut ThreadRng) {
    let mut comps = COMPETITIONS.lock().unwrap().clone();
    for comp in comps.values_mut() {
        let mut season = Season::fetch_from_db(&comp.id, comp.get_seasons_amount() - 1);

        // Simulate all games that happen today.
        if comp.format.is_some() {
            season.simulate_day(&comp, &today, rng);
        }

        // Create new seasons for parent competitions whose seasons are over.
        if comp.parent_comp_id == 0 && *today > db_string_to_date(&season.end_date) {
            // Cannot change teams between seasons, for now.
            let teams: Vec<TeamId> = season.teams.iter().map(|a | a.team_id).collect();
            comp.create_and_setup_seasons(&teams, today, rng);
        }
    }
}

// Do the daily tasks of managers (and teams, they are connected).
fn handle_managers_and_teams(countries: &HashMap<CountryId, Country>, today: &Date, rng: &mut ThreadRng) {
    let mut managers = MANAGERS.lock().unwrap().clone();
    let mut teams_visited = HashSet::new();

    for manager in managers.values_mut() {
        // TODO: have the manager look for a job or something.
        if manager.person.contract.is_none() {
            continue;
        }

        let mut team = Team::fetch_from_db(&manager.person.contract.as_ref().unwrap().team_id);
        teams_visited.insert(team.id);

        // Initial evaluation here.
        // Done for human managers as well so the players can evaluate the contract offers they receive.
        team.evaluate_player_needs();

        // Do not do anything on behalf of the human.
        if manager.is_human {
            team.return_actions_to_full();
            team.season_end_checker(countries, today, rng);
            team.save();
            continue;
        }


        let mut has_changes = false;
        while team.actions_remaining > 0 {
            let contract_offered = team.offer_contract(today, rng);
            if !contract_offered {
                break;
            }
            else {
                has_changes = true;
            }
        }

        // Another evaluation used for players' decision-making.
        if has_changes {
            team.evaluate_player_needs();
        }

        team.return_actions_to_full();
        team.season_end_checker(countries, today, rng);
        team.save();
    }

    // TODO: teams without managers should do tasks specific to them.
    /*
    let mut teams = TEAMS.lock().unwrap().clone();
    for team in teams.values_mut() {

    }
    */
}

// Do the daily tasks of players.
fn handle_players(today: &Date, rng: &mut ThreadRng) {
    let mut players: Vec<Player> = PLAYERS.lock().unwrap().iter().filter_map(|(_, a)| match a.person.is_active {
        true => Some(a.clone()),
        _ => None,
    }).collect();

    for player in players.iter_mut() {
        // Check if the player's contract has expired.
        let expired = player.person.check_if_contract_expired(today);
        if expired {
            let mut team = Team::fetch_from_db(&player.person.contract.as_ref().unwrap().team_id);
            player.person.contract = None;

            team.roster.retain(|id| *id != player.id);
            team.save();
        }

        player.check_expired_offers(today);

        let signs_contract = player.person.decide_to_sign(today, rng);
        if signs_contract {
            player.choose_contract(today, rng);
        }

        // Player thinks about retiring
        if player.retires(rng) {
            player.reject_contracts();
            player.person.is_active = false;
            continue;
        }

        // Training after choosing the contract sounds most fair,
        // as then the player will choose their contract based on the most recent
        // information available to the managers, both human and AI.
        player.daily_training(today, rng);
        player.save();
    }
}