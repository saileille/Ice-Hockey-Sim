// Methods for generating match schedules for Stage.
use std::collections::{HashMap, HashSet};
use std::cmp::Ordering;
use std::ops::Range;
use rand::{rng, Rng};
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;

use clearscreen;

use crate::team::Team;

use super::{Stage, TeamData};

#[derive(Default, Clone)]
struct TeamScheduleData {
    team_id: usize,
    home_matches: HashSet<usize>,   // Contains teams that the team plays against home.
    away_matches: HashSet<usize>,   // Contains teams that the team plays against away.
}

// Methods
impl TeamScheduleData {
    fn get_home_match_count(&self, prev: &Self) -> u8 {
        (self.home_matches.len() + prev.home_matches.len()) as u8
    }
    
    fn get_away_match_count(&self, prev: &Self) -> u8 {
        (self.away_matches.len() + prev.away_matches.len()) as u8
    }
    
    // Add home and away matches together.
    fn get_match_count(&self, prev: &Self) -> u8 {
        self.get_home_match_count(prev) + self.get_away_match_count(prev)
    }

    // Check if the team can have any more home games.
    fn can_have_home_games(&self, prev: &Self, matches: u8) -> bool {
        let total_matches: u8 = matches + prev.get_match_count( &Self::default());
        self.get_home_match_count(prev) < (total_matches + 1) / 2
    }

    // Check if the team can have any more away games.
    fn can_have_away_games(&self, prev: &Self, matches: u8) -> bool {
        let total_matches: u8 = matches + prev.get_match_count( &Self::default());
        self.get_away_match_count(prev) < (total_matches + 1) / 2
    }

    // Get a combined HashSet of home_matches and away_matches.
    fn get_all_opponents(&self) -> HashSet<usize> {
        // Using bitwise-or to merge the HashSets.
        &self.home_matches | &self.away_matches
    }

    // Get the difference between home and away matches.
    // Positive values indicate there are more home matches.
    // Negative values indicate there are more away matches.
    fn get_home_away_difference(&self, prev: &Self) -> i8 {
        (self.get_home_match_count(prev) as i8) - (self.get_away_match_count(prev) as i8)
    }

    // Get the percentage of disproportion between home and away matches.
    // Positive values indicate there are more home matches.
    // Negative values indicate there are more away matches.
    fn get_home_away_difference_ratio(&self, prev: &Self) -> f64 {
        // Avoiding divide-by-zero.
        let match_count: f64 = self.get_match_count(prev) as f64;
        if match_count == 0.0 { return 0.0; }

        let diff: f64 = self.get_home_away_difference(prev) as f64;
        return diff / match_count;
    }

    // Check that the team has enough matches, and that home and away matches are balanced.
    fn is_valid_schedule(&self, matches: u8) -> bool {
        let home_count: u8 = self.get_home_match_count(&TeamScheduleData::default());
        let away_count: u8 = self.get_away_match_count(&TeamScheduleData::default());
        let total_count: u8 = home_count + away_count;

        total_count == matches &&
        (home_count as i8 - away_count as i8).abs() <= 1
    }

    // Check if the schedule data is full (no more matches can be inserted).
    fn is_full(&self, matches: u8) -> bool {
        self.get_match_count(&TeamScheduleData::default()) >= matches
    }

    fn get_debug_info(&self, prev: &TeamScheduleData) -> String {
        format!("{}\nHome: {}\nAway: {}", self.team_id, self.get_home_match_count(prev), self.get_away_match_count(prev))
    }
}

// Static
impl TeamScheduleData {
    // Check that everyone has a valid schedule.
    fn is_valid_schedule_for_all(schedule_data: &Vec<Self>, matches: u8) -> bool {
        for team in schedule_data.iter() {
            if !team.is_valid_schedule(matches) { return false; }
        }

        return true;
    }

    // Sort the schedule data so that the team most needing a home/away match is at the top.
    fn sort_default(schedule_data: &mut Vec<Self>, prev_schedule_map: &HashMap<usize, Self>) {
        schedule_data.sort_by(|a: &Self, b: &Self| {
            let prev_a: &Self = match prev_schedule_map.get(&a.team_id) {
                Some(prev_ref) => prev_ref,
                None => &TeamScheduleData::default(),
            };

            let prev_b: &Self = match prev_schedule_map.get(&b.team_id) {
                Some(prev_ref) => prev_ref,
                None => &TeamScheduleData::default(),
            };

            let a_ratio: f64 = a.get_home_away_difference_ratio(prev_a).abs();
            let b_ratio: f64 = b.get_home_away_difference_ratio(prev_b).abs();

            if b_ratio < a_ratio { return Ordering::Less; }
            else if b_ratio > a_ratio { return Ordering::Greater }

            let a_diff: u8 = a.get_home_away_difference(prev_a).abs() as u8;
            let b_diff: u8 = b.get_home_away_difference(prev_b).abs() as u8;

            if b_diff < a_diff { return Ordering::Less }
            else if b_diff > a_diff { return Ordering::Greater }

            let (a_total, b_total) = (a.get_match_count(prev_a), b.get_match_count(prev_b));
            if a_total < b_total { return Ordering::Less; }
            else if a_total > b_total { return Ordering::Greater; }

            return Ordering::Equal;
        });
    }

    // Sort the schedule data so that the team most needing a home match is at the top.
    fn sort_home(schedule_data: &mut Vec<Self>, prev_schedule_map: &HashMap<usize, Self>) {
        schedule_data.sort_by(|a: &Self, b: &Self| {
            let prev_a: &Self = match prev_schedule_map.get(&a.team_id) {
                Some(prev_ref) => prev_ref,
                None => &TeamScheduleData::default(),
            };

            let prev_b: &Self = match prev_schedule_map.get(&b.team_id) {
                Some(prev_ref) => prev_ref,
                None => &TeamScheduleData::default(),
            };

            let a_ratio: f64 = a.get_home_away_difference_ratio(prev_a);
            let b_ratio: f64 = b.get_home_away_difference_ratio(prev_b);

            if b_ratio < a_ratio { return Ordering::Less; }
            else if b_ratio > a_ratio { return Ordering::Greater }

            let a_diff: i8 = a.get_home_away_difference(prev_a);
            let b_diff: i8 = b.get_home_away_difference(prev_b);

            if b_diff < a_diff { return Ordering::Less }
            else if b_diff > a_diff { return Ordering::Greater }

            let (a_total, b_total) = (a.get_match_count(prev_a), b.get_match_count(prev_b));
            if a_total < b_total { return Ordering::Less }
            else if b_total < a_total { return Ordering::Greater }

            return Ordering::Equal;
        });
    }

    // Sort the schedule data so that the team most needing an away match is at the top.
    fn sort_away(schedule_data: &mut Vec<Self>, prev_schedule_map: &HashMap<usize, Self>) {
        schedule_data.sort_by(|a: &Self, b: &Self| {
            let prev_a: &Self = match prev_schedule_map.get(&a.team_id) {
                Some(prev_ref) => prev_ref,
                None => &TeamScheduleData::default(),
            };

            let prev_b: &Self = match prev_schedule_map.get(&b.team_id) {
                Some(prev_ref) => prev_ref,
                None => &TeamScheduleData::default(),
            };

            let a_ratio: f64 = a.get_home_away_difference_ratio(prev_a);
            let b_ratio: f64 = b.get_home_away_difference_ratio(prev_b);

            if a_ratio < b_ratio { return Ordering::Less; }
            else if a_ratio > b_ratio { return Ordering::Greater }

            let a_diff: i8 = a.get_home_away_difference(prev_a);
            let b_diff: i8 = b.get_home_away_difference(prev_b);

            if a_diff < b_diff { return Ordering::Less }
            else if a_diff > b_diff { return Ordering::Greater }

            let (a_total, b_total) = (a.get_match_count(prev_a), b.get_match_count(prev_b));
            if a_total < b_total { return Ordering::Less }
            else if b_total < a_total { return Ordering::Greater }

            return Ordering::Equal;
        });
    }

    // Get a new schedule_data vector with only teams that can play away games.
    fn filter_for_home_game(schedule_data: &Vec<Self>, prev_schedule_map: &HashMap<usize, Self>, matches: u8) -> Vec<Self> {
        let mut filtered: Vec<Self> = Vec::new();
        for item in schedule_data {
            let prev: &TeamScheduleData = match prev_schedule_map.get(&item.team_id) {
                Some(prev_ref) => prev_ref,
                None => &TeamScheduleData::default(),
            };

            if item.can_have_away_games(prev, matches) {
                filtered.push(item.clone())
            };
        }
        return filtered;
    }

    // Get a new schedule_data vector with only teams that can play home games.
    fn filter_for_away_game(schedule_data: &Vec<Self>, prev_schedule_map: &HashMap<usize, Self>, matches: u8) -> Vec<Self> {
        let mut filtered: Vec<Self> = Vec::new();
        for item in schedule_data {
            let prev: &TeamScheduleData = match prev_schedule_map.get(&item.team_id) {
                Some(prev_ref) => prev_ref,
                None => &TeamScheduleData::default(),
            };

            if item.can_have_home_games(prev, matches) {
                filtered.push(item.clone())
            };
        }
        return filtered;
    }

    // Move the schedule_data that is completed.
    fn move_completed(schedule_data: &mut Vec<Self>, completed: &mut Vec<Self>, matches: u8)  {
        let mut index: usize = 0;
        while index < schedule_data.len() {
            let data: &Self = &schedule_data[index];
            if data.is_full(matches) {
                completed.push(data.clone());
                schedule_data.remove(index);
            }
            else {
                index += 1;
            }
        }
    }

    // Generate schedule data objects.
    fn generate(stage_teams: &Vec<TeamData>) -> Vec<Self> {
        let mut schedule_data: Vec<Self> = Vec::new();
        for team_data in stage_teams.iter() {
            schedule_data.push(Self {
                team_id: team_data.team_id,
                home_matches: HashSet::new(),
                away_matches: HashSet::new(),
            })
        }

        return schedule_data;
    }

    // Convert the vector of TeamScheduleData to hashmap. Consume the vector in the process.
    fn vector_to_hashmap(schedule_data: Vec<Self>) -> HashMap<usize, Self> {
        let mut map: HashMap<usize, Self> = HashMap::new();
        for item in schedule_data {
            map.insert(item.team_id, item);
        }
        return map;
    }

    fn get_debug_info_all(schedule_data: &Vec<Self>, prev_schedule_map: &HashMap<usize, Self>) {
        let mut log: String = String::new();
        for team in schedule_data {
            let prev: &TeamScheduleData = match prev_schedule_map.get(&team.team_id) {
                Some(p) => p,
                None => &TeamScheduleData::default(),
            };

            log += &team.get_debug_info(prev);
            log += "\n\n";
        }

        println!("{}", log.trim_end_matches("\n").to_string());
    }
}

// Functional
impl Stage {
    // Generate a match schedule for round robin stages.
    pub fn matches_for_round_robin(&self) {
        let matchups: Vec<[usize; 2]> = self.generate_round_robin_matches();
        let matchdays: Vec<Vec<[usize; 2]>> = Vec::new();
    }

    // Generate a single matchday.
    fn generate_matchday(&self, matchups: &mut Vec<[usize; 2]>) -> Vec<[usize; 2]> {
        let mut valid_matches: Vec<[usize; 2]> = matchups.clone();
        let mut rng: ThreadRng = rand::rng();
        let mut matchday: Vec<[usize; 2]> = Vec::new();
        
        while valid_matches.len() > 0 {
            let index: usize = rng.random_range(Range {start: 0, end: valid_matches.len()});
            let game: [usize; 2] = valid_matches[index].clone();
            matchday.push(game.clone());
            matchups.remove(index);

            // Remove all matches from valid_matches where either of the teams play.
            valid_matches.retain(|g: &[usize; 2]| !g.contains(&game[0]) && !g.contains(&game[1]));
        }

        return matchday;
    }

    // Generate matches where every team plays every other home and away.
    fn generate_possible_matches(&self, matchups: &mut Vec<[usize; 2]>) {
        let team_ids: Vec<usize> = self.get_team_ids();

        for home_id in team_ids.iter() {
            for away_id in team_ids.iter() {
                if home_id != away_id { matchups.push([home_id.clone(), away_id.clone()]) }
            }
        }
    }

    // Generate matches for a round robin stage.
    fn generate_round_robin_matches(&self) -> Vec<[usize; 2]> {
        let rounds: u8 = self.round_robin_rules.rounds;
        let matches_in_round: u8 = (self.teams.len() - 1) as u8;
        let mut matches: u8 = rounds * matches_in_round + self.round_robin_rules.extra_matches;
        let mut matchups: Vec<[usize; 2]> = Vec::new();

        // Complete rounds.
        while matches >= matches_in_round * 2 {
            self.generate_possible_matches(&mut matchups);
            matches -= matches_in_round * 2;
        }

        // Partial rounds.
        
        let mut prev_schedule_data: Vec<TeamScheduleData> = Vec::new();
        if matches >= matches_in_round {
            prev_schedule_data = self.generate_incomplete_round(matches_in_round, &mut matchups, prev_schedule_data);
            matches -= matches_in_round;
        }

        if matches > 0 {
            self.generate_incomplete_round(matches, &mut matchups, prev_schedule_data);
        }

        return matchups;
    }

    // Generate matches as an incomplete round.
    // Return the resulting schedule data.
    fn generate_incomplete_round(
        &self, matches_per_team: u8,
        matchups: &mut Vec<[usize; 2]>,
        prev_schedule_data: Vec<TeamScheduleData>) -> Vec<TeamScheduleData> {
    // -----------------------------------------------------------
        let mut schedule_data: Vec<TeamScheduleData> = TeamScheduleData::generate(&self.teams);
        let mut completed_schedule_data: Vec<TeamScheduleData> = Vec::new();
        let mut created_matchups: Vec<[usize; 2]> = Vec::new();

        let prev_schedule_map: HashMap<usize, TeamScheduleData> = TeamScheduleData::vector_to_hashmap(prev_schedule_data);

        let mut rng: ThreadRng = rng();

        // Variable success measures whether the creation of the match schedule was successful or not.
        let mut success: bool = true;
        while schedule_data.len() > 0 {
            // Randomise and sort.
            schedule_data.shuffle(&mut rng);
            TeamScheduleData::sort_default(&mut schedule_data, &prev_schedule_map);
            let mut temp_schedule_data: Vec<TeamScheduleData> = schedule_data.clone();

            let mut team1: TeamScheduleData = temp_schedule_data[0].clone();
            temp_schedule_data.remove(0);

            // Remove every item in temp_schedule_data that already plays against team1.
            let blacklist: HashSet<usize> = team1.get_all_opponents();
            temp_schedule_data.retain(|data: &TeamScheduleData| !blacklist.contains(&data.team_id));

            if temp_schedule_data.len() == 0 {
                success = false;
                break;
            }

            let home_filter: Vec<TeamScheduleData> = TeamScheduleData::filter_for_home_game(&temp_schedule_data, &prev_schedule_map, matches_per_team);
            let away_filter: Vec<TeamScheduleData> = TeamScheduleData::filter_for_away_game(&temp_schedule_data, &prev_schedule_map, matches_per_team);

            if home_filter.len() == 0 && away_filter.len() == 0 {
                success = false;
                break;
            }
            
            let prev_team1: &TeamScheduleData = match prev_schedule_map.get(&team1.team_id) {
                Some(p) => p,
                None => &TeamScheduleData::default(),
            };

            let home_away_ratio: f64 = team1.get_home_away_difference_ratio(prev_team1);
            let mut team2: TeamScheduleData;

            // team1 needs a home game.
            if away_filter.len() == 0 || (home_filter.len() > 0 && home_away_ratio <= 0.0) {
                temp_schedule_data = home_filter;
                TeamScheduleData::sort_away(&mut temp_schedule_data, &prev_schedule_map);
                team2 = temp_schedule_data[0].clone();
                created_matchups.push([team1.team_id, team2.team_id]);

                team1.home_matches.insert(team2.team_id);
                team2.away_matches.insert(team1.team_id);
            }

            // team1 needs an away game.
            else {
                temp_schedule_data = away_filter;
                TeamScheduleData::sort_home(&mut temp_schedule_data, &prev_schedule_map);
                team2 = temp_schedule_data[0].clone();
                created_matchups.push([team2.team_id, team1.team_id]);
                
                team1.away_matches.insert(team2.team_id);
                team2.home_matches.insert(team1.team_id);
            }

            // Update schedule_data with changed data.
            for item in schedule_data.iter_mut() {
                if item.team_id == team1.team_id { *item = team1.clone(); }
                else if item.team_id == team2.team_id { *item = team2.clone(); }
            }

            // clearscreen::clear().unwrap();
            // TeamScheduleData::get_debug_info_all(&schedule_data, &prev_schedule_map);
            // println!("---------------------------------------------------------------------------------------");

            // Move the teams away that cannot have any more matches.
            TeamScheduleData::move_completed(&mut schedule_data, &mut completed_schedule_data, matches_per_team);
        }

        // Add created_matchups to matchups here.
        println!("{success}");
        if success {
            matchups.append(&mut created_matchups);
        }

        return schedule_data;
    }
}