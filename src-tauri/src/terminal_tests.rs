// Functions for testing things on the terminal.
use std::{collections::HashMap, time::Instant};
use time::Date;

use crate::{
    database::{COMPETITIONS, TODAY},
    commands::go_to_next_day,
    competition::{Competition, stage::Stage,}
};

pub fn test_comp_generation() {
    let start_time: Instant = Instant::now();

    let mut comps: HashMap<u8, Competition> = COMPETITIONS.lock().unwrap().clone();
    for comp in comps.values_mut() {
        comp.setup();
    }

    let comp: &Competition = comps.get(&1).unwrap();
    let end_date: Date = Stage::fetch_from_db(&comp.stage_ids[0]).get_next_end_date();

    loop {
        go_to_next_day();
        if *TODAY.lock().unwrap() > end_date { break; }
    }

    let stage: Stage = Stage::fetch_from_db(&comp.stage_ids[0]);
    println!("{}", stage.display_standings());
    println!("{}", stage.display_match_schedule());

    println!("Completed in {} seconds", start_time.elapsed().as_secs());
}

/* pub fn test_match_generator() {
    let sort_types: [MatchGenType; 3] = [MatchGenType::MatchCount, MatchGenType::Random, MatchGenType::Alternating];

    let mut counter: usize = 0;
    let benchmark: Instant = Instant::now();
    for sort_team1 in sort_types.iter() {

        for sort_team2 in sort_types.iter() {
            let mut fail_counter: usize = 0;
            let mut skip_counter: usize = 0;
            let sort_type_benchmark: Instant = Instant::now();

            for team_count in 3..=32 {
                let mut teams: Vec<TeamId> = Vec::new();
                for id in (Range {start: 0, end: team_count}) {
                    teams.push(id);
                }

                for match_count in (Range {start: 1, end: team_count * 2 - 2}) {
                    let mut stage: Stage = Stage::build(
                        "blbl",
                        teams.clone(),
                        rules::RoundRobin::build(0, match_count as u8, sort_team1.clone(), sort_team2.clone()),
                    );

                    // Let's skip what is doomed to fail.
                    if !stage.has_valid_match_amount() { continue; }

                    stage.generate_schedule_for_round_robin();

                    skip_counter += (match_count - stage.get_matches_per_team()) as usize;
                    fail_counter += stage.failures;
                    counter += 1;
                }
            }
            println!("{sort_team1:?}+{sort_team2:?}: {skip_counter} skips and {fail_counter} failures in {:.0} seconds.", sort_type_benchmark.elapsed().as_secs_f64());
        }
    }

    let seconds = benchmark.elapsed().as_secs_f64();

    println!("\nCreated {counter} match schedules in {seconds:.0} seconds ({:.2} seconds per 100 schedules.)", seconds / (counter as f64 / 100.0));
}*/