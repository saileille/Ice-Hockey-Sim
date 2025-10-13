
// Functions for testing things on the terminal.
use std::{collections::HashMap, time::Instant};
use time::Date;

use crate::{
    commands::go_to_next_day, competition::{season::Season, Competition}, database::{COMPETITIONS, SEASONS, TEAMS, TODAY}, time::{date_to_db_string, db_string_to_date}, types::CompetitionId
};

pub fn test_comp_generation() {
    let start_time = Instant::now();

    // Give teams their lineups.
    let mut teams = TEAMS.lock().unwrap().clone();
    for team in teams.values_mut() {
        team.setup(0, 0);
    }

    let liiga_id = 1;
    let regular_season_id = 2;
    let playoffs_id = 3;

    let mut regular_season = Season::fetch_from_db(&regular_season_id, 0);
    let mut end_date = db_string_to_date(&regular_season.end_date);

    // Simulate the regular season.
    loop {
        go_to_next_day();
        if *TODAY.lock().unwrap() > end_date { break; }
    }

    regular_season = Season::fetch_from_db(&regular_season_id, 0);
    println!("{}", regular_season.display_standings());

    // let regular_season_comp = Competition::fetch_from_db(&regular_season_id).unwrap();
    // println!("\n{}", regular_season.display_match_schedule(&regular_season_comp));

    let mut playoffs = Season::fetch_from_db(&playoffs_id, 0);
    end_date = db_string_to_date(&playoffs.end_date);

    // println!("{:#?}", SEASONS.lock().unwrap().get(&playoffs_id).unwrap());

    loop {
        go_to_next_day();
        if *TODAY.lock().unwrap() > end_date { break; }
    }

    playoffs = Season::fetch_from_db(&playoffs_id, 0);
    let playoffs_comp = Competition::fetch_from_db(&playoffs_id).unwrap();

    println!("\n{}", playoffs.display_match_schedule(&playoffs_comp));

    println!("\nCompleted in {} seconds", start_time.elapsed().as_secs());
}