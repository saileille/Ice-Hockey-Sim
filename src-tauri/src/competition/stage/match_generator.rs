// Scheduling-related methods that are valid for both Knockout and RoundRobin.

use std::ops::Range;
use rand::{rng, rngs::ThreadRng, Rng};
use ::time::Date;

use crate::{
    competition::stage::knockout::{KnockoutRound, KnockoutPair},
    match_event::Game,
    time::{date_to_db_string, get_dates},
    types::{StageId, TeamId}
};

// Give each matchday a date and add to the stage's schedule.
pub fn assign_dates_for_matchdays(matchdays: &Vec<Vec<[TeamId; 2]>>, start_date: &Date, end_date: &Date, stage_id: StageId) {
    let mut dates: Vec<Date> = get_dates(start_date, end_date);
    let mut rng: ThreadRng = rng();

    for matchday in matchdays.iter() {
        let date: Date = dates.swap_remove(rng.random_range(Range {start: 0, end: dates.len()}));
        let date_string: String = date_to_db_string(&date);
        build_and_save_games(matchday, &date_string, stage_id);
    }
}

// Convert the simple representations of two teams into Game elements, and save them to the database.
fn build_and_save_games(match_pool: &Vec<[TeamId; 2]>, date: &str, stage_id: StageId) {
    for matchup in match_pool {
        Game::build_and_save(matchup[0], matchup[1], stage_id, date);
    }
}

// Generate a single matchday.
// Attempts to make as many teams as possible to play at the same time.
fn generate_matchday(match_pool: &mut Vec<[TeamId; 2]>, rng: &mut ThreadRng) -> Vec<[TeamId; 2]> {
    let mut valid_matches: Vec<[TeamId; 2]> = match_pool.clone();
    let mut matchday: Vec<[TeamId; 2]> = Vec::new();

    while !valid_matches.is_empty() {
        let game: [TeamId; 2] = valid_matches.swap_remove(rng.random_range(Range {start: 0, end: valid_matches.len()}));
        matchday.push(game.clone());

        // Remove all matches from valid_matches where either of the teams play.
        valid_matches.retain(|g: &[TeamId; 2]| !g.contains(&game[0]) && !g.contains(&game[1]));

        // Removing the match from the match pool.
        match_pool.remove(
            match_pool.iter()
            .position(|g| *g == game).unwrap());
    }

    return matchday;
}

// Generate individual matchdays from the given list of games.
pub fn generate_matchdays(match_pool: &mut Vec<[TeamId; 2]>) -> Vec<Vec<[TeamId; 2]>> {
    let mut matchdays: Vec<Vec<[TeamId; 2]>> = Vec::new();
    let mut rng: ThreadRng = rand::rng();
    while match_pool.len() > 0 {
        matchdays.push(generate_matchday(match_pool, &mut rng));
    }
    return matchdays;
}

impl KnockoutRound {
    // Generate a match pool for the knockout round.
    pub fn generate_match_pool(&self) -> Vec<[TeamId; 2]> {
        let mut match_pool: Vec<[u8; 2]> = Vec::new();

        for pair in self.pairs.iter() {
            pair.generate_match_pool(self.get_maximum_matches_in_pair(), &mut match_pool);
        }

        return match_pool;
    }
}

impl KnockoutPair {
    // Generate a match pool for the knockout pair.
    fn generate_match_pool(&self, match_count: u8, match_pool: &mut Vec<[TeamId; 2]>) {
        for i in (Range {start: 0, end: match_count}) {
            if i % 2 == 0 {
                match_pool.push([self.home.team_id, self.away.team_id]);
            }
            else {
                match_pool.push([self.away.team_id, self.home.team_id]);
            }
        }
    }
}