mod match_generator;
mod sorting;

use std::{collections::HashMap, ops::Range};
use rand::{rng, rngs::ThreadRng, Rng};
use ::time::Date;

use crate::{
    types::TeamId,
    time::{get_dates, date_to_db_string},
    match_event::Game
};

use super::{Stage, round_robin::RoundRobin};


impl RoundRobin {
    fn calc_matches_in_match_pool(&self, match_pool: &Vec<[TeamId; 2]>, stage: &Stage) {
        let mut teams: HashMap<TeamId, u8> = HashMap::new();
        for id in stage.teams.keys() {
            teams.insert(*id, 0);
        }
        for game in match_pool.iter() {
            for team in game.iter() {
                *teams.get_mut(team).unwrap() += 1;
            }
        }

        println!("{teams:#?}");
        panic!();

    }
    fn calc_matches_in_matchdays(&self, matchdays: &Vec<Vec<[TeamId; 2]>>, stage: &Stage) {
        let mut teams: HashMap<TeamId, u8> = HashMap::new();
        for id in stage.teams.keys() {
            teams.insert(*id, 0);
        }
        for matchday in matchdays.iter() {
            for game in matchday.iter() {
                for team in game.iter() {
                    *teams.get_mut(team).unwrap() += 1;
                }
            }
        }

        println!("{teams:#?}");
    }

    // Generate a match schedule for round robin stages.
    pub fn generate_schedule(&self, stage: &Stage) {
        let mut match_pool: Vec<[TeamId; 2]> = self.generate_round_robin_matches(stage);
        let matchdays: Vec<Vec<[TeamId; 2]>> = Stage::generate_matchdays(&mut match_pool);
        stage.assign_dates_for_matchdays(&matchdays);
    }
}

// Scheduling-related methods that are valid for both Knockout and RoundRobin.
impl Stage {
    // Give each matchday a date and add to the stage's schedule.
    fn assign_dates_for_matchdays(&self, matchdays: &Vec<Vec<[TeamId; 2]>>) {
        let mut dates: Vec<Date> = get_dates(&self.get_next_start_date(), &self.get_next_end_date());
        let mut rng: ThreadRng = rng();

        for matchday in matchdays.iter() {
            let index: usize = rng.random_range(Range {start: 0, end: dates.len()});
            let date_string: String = date_to_db_string(&dates[index]);
            self.build_and_save_games(matchday, &date_string);
            dates.remove(index);
        }
    }

    // Convert the simple representations of two teams into Game elements, and save them to the database.
    fn build_and_save_games(&self, match_pool: &Vec<[TeamId; 2]>, date: &str) {
        for matchup in match_pool {
            Game::build_and_save(matchup[0], matchup[1], self.id, date);
        }
    }

    // Generate a single matchday.
    // Attempts to make as many teams as possible to play at the same time.
    fn generate_matchday(match_pool: &mut Vec<[TeamId; 2]>, rng: &mut ThreadRng) -> Vec<[TeamId; 2]> {
        let mut valid_matches: Vec<[TeamId; 2]> = match_pool.clone();
        let mut matchday: Vec<[TeamId; 2]> = Vec::new();

        while valid_matches.len() > 0 {
            let index: usize = rng.random_range(Range {start: 0, end: valid_matches.len()});
            let game: [TeamId; 2] = valid_matches[index].clone();
            matchday.push(game.clone());

            // Remove all matches from valid_matches where either of the teams play.
            valid_matches.retain(|g: &[TeamId; 2]| !g.contains(&game[0]) && !g.contains(&game[1]));

            // Removing the match from the match pool.
            let mut i: usize = 0;
            loop {
                if game == match_pool[i] {
                    match_pool.remove(i);
                    break;
                }
                i += 1;
            }
        }

        return matchday;
    }

    // Generate individual matchdays from the given list of games.
    fn generate_matchdays(match_pool: &mut Vec<[TeamId; 2]>) -> Vec<Vec<[TeamId; 2]>> {
        let mut matchdays: Vec<Vec<[TeamId; 2]>> = Vec::new();
        let mut rng: ThreadRng = rand::rng();
        while match_pool.len() > 0 {
            matchdays.push(Stage::generate_matchday(match_pool, &mut rng));
        }
        return matchdays;
    }
}