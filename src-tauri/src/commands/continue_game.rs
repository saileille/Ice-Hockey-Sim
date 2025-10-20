// Commands and helper functions that have to do with continuing the game.

use std::collections::HashSet;

use time::Date;

use crate::{competition::season::Season, database::{COMPETITIONS, MANAGERS, PLAYERS, TODAY}, team::Team, time::{date_to_db_string, db_string_to_date}, types::TeamId};


// Advance the time with one day.
#[tauri::command]
pub fn go_to_next_day() -> String {
    let today = TODAY.lock().unwrap().clone();

    handle_managers_and_teams(&today);
    handle_players(&today);

    // Games are simulated here - this must be the last one!
    handle_comps(&today);

    *TODAY.lock().unwrap() = today.next_day().unwrap();
    return get_date_string();
}

// Get the current date as a string.
#[tauri::command]
pub fn get_date_string() -> String {
    date_to_db_string(&TODAY.lock().unwrap())
}

// Do the daily tasks of competitions.
fn handle_comps(today: &Date) {
    let mut comps = COMPETITIONS.lock().unwrap().clone();
    for comp in comps.values_mut() {
        let mut season = Season::fetch_from_db(&comp.id, comp.get_seasons_amount() - 1);

        // Simulate all games that happen today.
        if comp.format.is_some() {
            season.simulate_day(&comp, &today);
        }

        // Create new seasons for parent competitions whose seasons are over.
        if comp.parent_comp_id == 0 && *today > db_string_to_date(&season.end_date) {
            // Cannot change teams between seasons, for now.
            let teams: Vec<TeamId> = season.teams.iter().map(|a | a.team_id).collect();
            comp.create_and_setup_seasons(&teams);
        }
    }
}

// Do the daily tasks of managers (and teams, they are connected).
fn handle_managers_and_teams(today: &Date) {
    let mut managers = MANAGERS.lock().unwrap().clone();
    let mut teams_visited = HashSet::new();

    for manager in managers.values_mut() {
        // TODO: have the manager look for a job or something.
        if manager.person.contract.is_none() {
            continue;
        }

        let mut team = Team::fetch_from_db(&manager.person.contract.as_ref().unwrap().team_id);
        teams_visited.insert(team.id);

        // Do not do anything on behalf of the human.
        if manager.is_human {
            team.return_actions_to_full();
            team.save();
            continue;
        }

        // Evaluate player needs if there are none.
        if team.player_needs.is_empty() {
            team.evaluate_player_needs();
        }

        while team.actions_remaining > 0 {
            let contract_offered = team.offer_contract(today);
            if !contract_offered { break; }
        }

        team.return_actions_to_full();
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
fn handle_players(today: &Date) {
    let mut players = PLAYERS.lock().unwrap().clone();
    for player in players.values_mut() {
        let mut has_changes = false;

        // Check if the player's contract has expired.
        if player.person.check_if_contract_expired() {
            let mut team = Team::fetch_from_db(&player.person.contract.as_ref().unwrap().team_id);
            player.person.contract = None;

            team.roster.retain(|id| *id != player.id);
            team.save();

            has_changes = true;
        }

        // Check if any contract offers for the player have expired.
        let mut expired_indexes = Vec::new();
        for (i, offer) in player.person.contract_offers.iter().enumerate() {
            if offer.check_if_expired() {
                let mut team = Team::fetch_from_db(&offer.team_id);
                team.approached_players.retain(|id| *id != player.id);
                team.evaluate_player_needs();
                team.save();
                expired_indexes.push(i);
            }
        }

        if !expired_indexes.is_empty() {
            has_changes = true;
            for index in expired_indexes.iter().rev() {
                player.person.contract_offers.remove(*index);
            }
        }

        let signs_contract = player.person.decide_to_sign();
        if signs_contract {
            player.sign_contract(today);
            has_changes = true;
        }

        if has_changes { player.save(); }
    }
}