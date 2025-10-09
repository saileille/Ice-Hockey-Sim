// Functions exclusive to knockout stages.

use std::{collections::HashMap, ops::Range};
use rand::{rng, rngs::ThreadRng, Rng};
use ::time::{Date};

use crate::{
    competition::stage::match_generator::generate_matchdays, database::TODAY, match_event::Game, time::{date_to_db_string, db_string_to_date, get_dates, get_duration_from_days}, types::{convert, StageId, TeamId}
};
use super::{Stage, TeamStageData, match_generator::assign_dates_for_matchdays};

#[derive(Default, Clone)]
pub struct Knockout {
    // Wins required to progress in each round.
    // Starts from the beginning, the last value is duplicated
    // if there are more rounds than elements in this vector.
    wins_required: Vec<u8>,

    // The knockout structure.
    tree: Vec<KnockoutRound>,

    // Knockout rounds will continue until this many teams remain.
    teams_at_end: u8,
}

// Basics
impl Knockout {
    pub fn build(wins_required: Vec<u8>, teams_at_end: u8) -> Self {
        let mut knockout: Self = Self::default();
        knockout.wins_required = wins_required;
        knockout.teams_at_end = teams_at_end;
        return knockout;
    }

    // Make sure knockout rules do not have illegal values.
    pub fn is_valid(&self) -> bool {
        return !self.wins_required.is_empty();
    }

    // Set up the knockout stage.
    pub fn setup(&mut self, stage: &Stage) {
        self.create_tree(stage);
        self.set_date_boundaries_for_rounds(stage);

        let teams: Vec<TeamStageData> = stage.teams.values().cloned().collect();

        // Set up the first round.
        self.tree[0].setup(stage, &teams);
    }

    // Give each round's games a proportionate time window.
    // Makes sure that different rounds do not overlap.
    fn set_date_boundaries_for_rounds(&mut self, stage: &Stage) {
        let dates: Vec<Date> = get_dates(&stage.get_next_start_date(), &stage.get_next_end_date());
        let round_durations: Vec<u8> = self.get_round_durations(convert::usize_to_u8(dates.len()));

        // Starting from one day backwards.
        let mut last_date: Date = dates[0].checked_sub_std(get_duration_from_days(1)).unwrap();
        for (i, round) in self.tree.iter_mut().enumerate() {
            round.earliest_date = date_to_db_string(&last_date.next_day().unwrap());
            last_date = last_date.checked_add_std(get_duration_from_days(round_durations[i] as u64)).unwrap();
            round.latest_date = date_to_db_string(&last_date);
        }
    }

    // Get a duration for each round in the knockout stage.
    fn get_round_durations(&self, dates: u8) -> Vec<u8> {
        let matches_in_rounds: Vec<f64> = self.tree.iter().map(|a| a.get_maximum_matches_in_pair() as f64).collect();
        let total_matches: f64 = matches_in_rounds.iter().sum();

        // Minimum amount of dates in each round.
        let mut round_durations: Vec<u8> = matches_in_rounds.iter().map(|a| (a / total_matches * (dates as f64)) as u8).collect();

        // Calculating leftovers.
        let assigned_dates: u8 = round_durations.iter().sum();
        let mut leftover_dates: u8 = dates - assigned_dates;

        // Assign the leftovers randomly.
        let mut round_indexes: Vec<usize> = (Range {start: 0, end: self.tree.len()}).collect();
        let mut rng: ThreadRng = rng();
        while leftover_dates > 0 {
            let index: usize = round_indexes.swap_remove(rng.random_range(Range {start: 0, end: round_indexes.len()}));
            round_durations[index] += 1;
            leftover_dates -= 1;
        }

        return round_durations;
    }

    // Create a playoff tree.
    fn create_tree(&mut self, stage: &Stage) {
        let mut team_count: u8 = convert::usize_to_u8(stage.teams.len());
        while team_count > self.teams_at_end {
            let mut round: KnockoutRound = KnockoutRound::default();
            for _ in (Range {start: 0, end: team_count / 2}) {
                round.pairs.push(KnockoutPair::default());
            }

            // Add required wins for the round.
            match self.tree.len() < self.wins_required.len() {
                true => round.wins_required = self.wins_required[self.tree.len()],
                _ => round.wins_required = self.wins_required[self.wins_required.len() - 1]
            };

            round.assign_default_name();
            self.tree.push(round);
            team_count /= 2;
        }
    }

    // Get the ongoing knockout round's index.
    fn get_ongoing_round_index(&self, stage: &Stage) -> usize {
        let today = TODAY.lock().unwrap();
        for (i, round) in self.tree.iter().enumerate() {
            if *today >= db_string_to_date(&round.earliest_date) &&
            *today <= db_string_to_date(&round.latest_date) {
                return i;
            }
        }

        panic!("Why did you call me? Today is {}. I am active from {:?} to {:?}", date_to_db_string(&today), stage.earliest_date, stage.latest_date);
    }

    // Check if the knockout stage has ended.
    fn has_ended(&mut self, stage: &Stage) -> bool {
        let mut index: usize = self.get_ongoing_round_index(stage);
        let mut current_round: KnockoutRound = self.tree[index].clone();

        // Check that we have not already done this.
        if current_round.is_over() { return false; }

        let over: bool = current_round.handle_played_pairs(stage.id);
        if !over {
            self.tree[index] = current_round;
            return false;
        }

        index += 1;
        if self.tree.len() <= index {
            self.tree[index] = current_round;
            return true;
        }

        self.tree[index].setup(stage, &current_round.advanced_teams);
        return false;
    }
}

#[derive(Clone, Default)]
pub struct KnockoutRound {
    name: String,
    pub pairs: Vec<KnockoutPair>,
    earliest_date: String,
    latest_date: String,
    wins_required: u8,
    advanced_teams: Vec<TeamStageData>,
    eliminated_teams: Vec<TeamStageData>,
}

impl KnockoutRound {
    fn build(wins_required: u8) -> Self {
        let mut round: Self = Self::default();
        round.wins_required = wins_required;
        return round;
    }

    // Set up a knockout round.
    fn setup(&mut self, stage: &Stage, teams: &Vec<TeamStageData>) {
        self.draw_teams(teams);
        let mut match_pool: Vec<[u8; 2]> = self.generate_match_pool();
        let matchdays: Vec<Vec<[u8; 2]>> = generate_matchdays(&mut match_pool);
        assign_dates_for_matchdays(&matchdays, &db_string_to_date(&self.earliest_date), &db_string_to_date(&self.latest_date), stage.id);
    }

    // Get the amount of matches there can be in the round at most.
    pub fn get_maximum_matches_in_pair(&self) -> u8 {
        return self.wins_required * 2 - 1
    }

    // Get a name for the knockout round based on how many pairs there are.
    // Currently the only way to assign a name for it, should be changed later.
    fn assign_default_name(&mut self) {
        match self.pairs.len() {
            4 => self.name = "Quarter Final".to_string(),
            2 => self.name = "Semi Final".to_string(),
            1 => self.name = "Final".to_string(),
            _ => return
        }
    }

    // Draw the pairs for the round.
    fn draw_teams(&mut self, teams: &Vec<TeamStageData>) {
        let mut pots: Vec<(u8, Vec<u8>)> = Self::create_pots(teams);

        let mut rng: ThreadRng = rng();
        for pair in self.pairs.iter_mut() {
            let last_index: usize = pots.len() - 1;
            let mut best_pot = pots[0].clone();
            let mut worst_pot = pots[last_index].clone();

            // Draw the teams for the pair.
            let home_id: TeamId = Self::draw_team(&mut best_pot.1, &mut rng);
            let away_id: TeamId = Self::draw_team(&mut worst_pot.1, &mut rng);

            pair.home = TeamStageData::build(home_id, best_pot.0);
            pair.away = TeamStageData::build(away_id, worst_pot.0);

            // Remove pots if empty.
            if best_pot.1.is_empty() { pots.remove(0); }
            else { pots[0] = best_pot; }

            if worst_pot.1.is_empty() { pots.remove(last_index); }
            else { pots[last_index] = worst_pot; }
        }
    }

    // Create pots from which to draw teams. Top seeds are first, bottom seeds are last.
    fn create_pots(teams: &Vec<TeamStageData>) -> Vec<(u8, Vec<TeamId>)> {
        let mut pots: Vec<(u8, Vec<TeamId>)> = Vec::new();
        for team in teams.iter() {
            match pots.iter().position(|pot| pot.0 == team.seed) {
                // Add team to an existing pot.
                Some(i) => pots[i].1.push(team.team_id),
                // Create a new pot if one does not exist.
                _ => pots.push((team.seed, vec![team.team_id]))
            }
        }

        // Sorting by seeds.
        pots.sort_by(|a, b| a.0.cmp(&b.0));
        return pots;
    }

    // Draw a team from the pot, and remove it from the pot.
    fn draw_team(pot: &mut Vec<TeamId>, rng: &mut ThreadRng) -> TeamId {
        pot.swap_remove(rng.random_range(Range {start: 0, end: pot.len()}))
    }

    // See if any of the round's pairs are done and handle them appropriately.
    // Return true if all pairs have been played.
    fn handle_played_pairs(&mut self, stage_id: StageId) -> bool {
        let mut round_done: bool = true;
        for pair in self.pairs.iter() {
            match pair.get_teams_and_handle_if_ended(stage_id, self.wins_required) {
                Some(teams) => {
                    // Assigning the teams to advanced and eliminated.
                    self.advanced_teams.push(teams.0);
                    self.eliminated_teams.push(teams.1);
                }
                _ => round_done = false
            }
        }

        return round_done;
    }

    // Quick way to check if the knockout round is over and has been dealt with already.
    fn is_over(&self) -> bool {
        return self.eliminated_teams.len() == self.pairs.len();
    }
}

#[derive(Default, Clone)]
pub struct KnockoutPair {
    pub home: TeamStageData,
    pub away: TeamStageData,
}

// Basics.
impl KnockoutPair {
    // Build the element.
    fn build(home: TeamStageData, away: TeamStageData) -> Self {
        Self {home: home, away: away}
    }

    // Get the victor and the loser of the pair, or None if neither has won.
    fn get_teams_if_ended(&self, wins_required: u8) -> Option<(TeamStageData, TeamStageData)> {
        if self.home.get_wins() >= wins_required {
            return Some((self.home.clone(), self.away.clone()));
        }
        if self.away.get_wins() >= wins_required {
            return Some((self.away.clone(), self.home.clone()));
        }

        return None;
    }

    // Check if the matchup has concluded, and handle it.
    fn get_teams_and_handle_if_ended(&self, stage_id: StageId, wins_required: u8) -> Option<(TeamStageData, TeamStageData)> {
        let teams: Option<(TeamStageData, TeamStageData)> = match self.get_teams_if_ended(wins_required) {
            Some(v) => Some(v),
            _ => return None
        };

        self.handle_ended(stage_id);
        return teams;
    }

    // Handle an ended knockout pair.
    fn handle_ended(&self, stage_id: StageId) {
        Game::remove_future_matches(stage_id, vec![self.home.team_id, self.away.team_id]);
    }
}