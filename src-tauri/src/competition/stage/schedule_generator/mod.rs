mod match_generator;
mod sorting;

use std::ops::Range;

use rand::{
    rng,
    Rng,
    seq::SliceRandom,
    rngs::ThreadRng
};

use crate::{
    types::{
        GameId, TeamId
    },
    match_event::Game
};

use super::Stage;

impl Stage {
    // Generate a match schedule for round robin stages.
    pub fn generate_schedule_for_round_robin(&mut self) {
        self.matchday_tests = Vec::new();
        let mut matchups: Vec<[TeamId; 2]> = self.generate_round_robin_matches();
        let mut matchdays: Vec<Vec<GameId>> = Vec::new();

        while matchups.len() > 0 {
            matchdays.push(self.generate_matchday(&mut matchups));
        }

        let mut rng: ThreadRng = rng();
        matchdays.shuffle(&mut rng);
        self.matchday_tests = matchdays;
    }

    // Generate a single matchday.
    // Attempts to make as many teams as possible to play at the same time.
    fn generate_matchday(&self, matchups: &mut Vec<[TeamId; 2]>) -> Vec<GameId> {
        let mut valid_matches: Vec<[TeamId; 2]> = matchups.clone();
        let mut rng: ThreadRng = rand::rng();
        let mut matchday: Vec<[TeamId; 2]> = Vec::new();

        while valid_matches.len() > 0 {
            let index: usize = rng.random_range(Range {start: 0, end: valid_matches.len()});
            let game: [TeamId; 2] = valid_matches[index].clone();
            matchday.push(game.clone());
            matchups.remove(index);

            // Remove all matches from valid_matches where either of the teams play.
            valid_matches.retain(|g: &[TeamId; 2]| !g.contains(&game[0]) && !g.contains(&game[1]));
        }

        return self.convert_to_games(matchups);
    }

    // Convert the simple representations of two teams into game IDs.
    fn convert_to_games(&self, matchups: &Vec<[TeamId; 2]>) -> Vec<GameId> {
        let mut games: Vec<GameId> = Vec::new();
        for matchup in matchups {
            let game: Game = Game::build_and_save(matchup[0], matchup[1], self.id);
            games.push(game.id);
        }

        return games;
    }
}