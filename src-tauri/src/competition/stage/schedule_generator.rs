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

// Compare functions and helpers.
impl TeamScheduleData {
    // Get the previous schedule datas for compare functions.
    fn get_previous(a: &Self, b: &Self, prev_schedule_map: &HashMap<usize, Self>) -> (Self, Self) {
        let prev_a: Self = match prev_schedule_map.get(&a.team_id) {
            Some(prev) => prev.clone(),
            None => Self::default(),
        };

        let prev_b: Self = match prev_schedule_map.get(&b.team_id) {
            Some(prev) => prev.clone(),
            None => Self::default(),
        };

        return (prev_a, prev_b);
    }

    // Compare the home-away difference. Team with more need for a home game comes first.
    fn compare_home_away(a: &Self, b: &Self, prev_a: &Self, prev_b: &Self) -> Ordering {
        let a_diff: i8 = a.get_home_away_difference(prev_a);
        let b_diff: i8 = b.get_home_away_difference(prev_b);

        return b_diff.cmp(&a_diff);
    }

    // Compare the home-away difference. Team with more need for an away game comes first.
    fn compare_away_home(a: &Self, b: &Self, prev_a: &Self, prev_b: &Self) -> Ordering {
        return Self::compare_home_away(a, b, prev_a, prev_b).reverse();
    }

    // Compare the absolute value of home-away difference.
    fn compare_home_away_abs(a: &Self, b: &Self, prev_a: &Self, prev_b: &Self) -> Ordering {
        let a_diff: i8 = a.get_home_away_difference(prev_a).abs();
        let b_diff: i8 = b.get_home_away_difference(prev_b).abs();

        return b_diff.cmp(&a_diff);
    }

    // Compare the match count.
    fn compare_match_count(a: &Self, b: &Self, prev_a: &Self, prev_b: &Self) -> Ordering {
        let (a_total, b_total) = (a.get_match_count(prev_a), b.get_match_count(prev_b));

        return (a_total as i8).cmp(&(b_total as i8));
    }

    // Sort the schedule data with criteria randomised. Let the chaos begin!
    fn sort_default_random(schedule_data: &mut Vec<Self>, prev_schedule_map: &HashMap<usize, Self>, rng: &mut ThreadRng) {
        let mut sort_functions: [fn(&Self, &Self, &Self, &Self) -> Ordering; 2] = [Self::compare_home_away_abs, Self::compare_match_count];
        sort_functions.shuffle(rng);

        schedule_data.sort_by(|a: &Self, b: &Self| {
            let (prev_a, prev_b) = Self::get_previous(a, b, prev_schedule_map);
            return sort_functions[0](a, b, &prev_a, &prev_b)
        });
    }

    // Sort the schedule data with criteria randomised. Let the chaos begin!
    fn sort_home_random(schedule_data: &mut Vec<Self>, prev_schedule_map: &HashMap<usize, Self>, rng: &mut ThreadRng) {
        let mut sort_functions: [fn(&Self, &Self, &Self, &Self) -> Ordering; 2] = [Self::compare_home_away, Self::compare_match_count];
        sort_functions.shuffle(rng);

        schedule_data.sort_by(|a: &Self, b: &Self| {
            let (prev_a, prev_b) = Self::get_previous(a, b, prev_schedule_map);
            return sort_functions[0](a, b, &prev_a, &prev_b)
        });
    }

    // Sort the schedule data with criteria randomised. Let the chaos begin!
    fn sort_away_random(schedule_data: &mut Vec<Self>, prev_schedule_map: &HashMap<usize, Self>, rng: &mut ThreadRng) {
        let mut sort_functions: [fn(&Self, &Self, &Self, &Self) -> Ordering; 2] = [Self::compare_away_home, Self::compare_match_count];
        sort_functions.shuffle(rng);

        schedule_data.sort_by(|a: &Self, b: &Self| {
            let (prev_a, prev_b) = Self::get_previous(a, b, prev_schedule_map);
            return sort_functions[0](a, b, &prev_a, &prev_b)
        });
    }

    // Sort the schedule data so that the team most needing a home/away match is at the top.
    fn sort_default(schedule_data: &mut Vec<Self>, prev_schedule_map: &HashMap<usize, Self>) {
        schedule_data.sort_by(|a: &Self, b: &Self| {
            let (prev_a, prev_b) = Self::get_previous(a, b, prev_schedule_map);

            return Self::compare_home_away_abs(a, b, &prev_a, &prev_b)
                .then(Self::compare_match_count(a, b, &prev_a, &prev_b));
        });
    }

    // Sort the schedule data so that the team most needing a home match is at the top.
    fn sort_home(schedule_data: &mut Vec<Self>, prev_schedule_map: &HashMap<usize, Self>) {
        schedule_data.sort_by(|a: &Self, b: &Self| {
            let (prev_a, prev_b) = Self::get_previous(a, b, prev_schedule_map);

            return Self::compare_home_away(a, b, &prev_a, &prev_b)
                .then(Self::compare_match_count(a, b, &prev_a, &prev_b));
        });
    }

    // Sort the schedule data so that the team most needing an away match is at the top.
    fn sort_away(schedule_data: &mut Vec<Self>, prev_schedule_map: &HashMap<usize, Self>) {
        schedule_data.sort_by(|a: &Self, b: &Self| {
            let (prev_a, prev_b) = Self::get_previous(a, b, prev_schedule_map);

            return Self::compare_home_away(a, b, &prev_a, &prev_b).reverse()
                .then(Self::compare_match_count(a, b, &prev_a, &prev_b));
        });
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
    pub fn matches_for_round_robin(&mut self) {
        let matchups: Vec<[usize; 2]> = self.generate_round_robin_matches();
        // let matchdays: Vec<Vec<[usize; 2]>> = Vec::new();
        self.match_tests = matchups;
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
    fn generate_full_round(&self, matchups: &mut Vec<[usize; 2]>) {
        let team_ids: Vec<usize> = self.get_team_ids();

        for home_id in team_ids.iter() {
            for away_id in team_ids.iter() {
                if home_id != away_id { matchups.push([*home_id, *away_id]) }
            }
        }
    }

    // Generate matches for a round robin stage.
    fn generate_round_robin_matches(&mut self) -> Vec<[usize; 2]> {
        // How many times should uncertain generations be attempted before giving up.
        const ATTEMPTS: u8 = u8::MAX;

        let matches_in_round: u8 = self.get_round_length();
        let matches_in_full_round: u8 = matches_in_round * 2;
        let mut matches: u8 = self.get_theoretical_matches_per_team();
        let mut matchups: Vec<[usize; 2]> = Vec::new();

        // Complete rounds.
        while matches >= matches_in_full_round {
            self.generate_full_round(&mut matchups);
            matches -= matches_in_full_round;
        }

        // Half rounds.
        let mut prev_schedule_data: Vec<TeamScheduleData> = Vec::new();
        if matches >= matches_in_round {
            prev_schedule_data = self.attempt_irregular_generation(matches_in_round, &mut matchups, prev_schedule_data, ATTEMPTS);
            
            // If unsuccessful, move on to the next part with one match less.
            if prev_schedule_data.len() == 0 {
                matches = matches_in_round - 1;

                // Making sure we are not trying the impossible.
                if self.teams.len() % 2 != 0 && matches % 2 != 0 {
                    matches -= 1;
                }
            }

            // Otherwise, just move on!
            else {
                matches -= matches_in_round;
            }
        }

        // Handle the leftover matches.
        while matches > 0 {
            prev_schedule_data = self.attempt_irregular_generation(matches, &mut matchups, prev_schedule_data, ATTEMPTS);
            
            // If unsuccessful, try again with one match less.
            if prev_schedule_data.len() == 0 {
                matches -= 1;
            }

            // Otherwise, exit the loop and then the method.
            else {
                break;
            }

            // Making sure we are not trying the impossible.
            if self.teams.len() % 2 != 0 && matches % 2 != 0 {
                matches -= 1;
            }
        }

        return matchups;
    }

    // Generate a match schedule with arbitrary number of games.
    // Add to an existing matchups vector if successful.
    // Return the schedule data. If unsuccessful, return empty vector.
    fn generate_irregular_matches(
        &self, matches_per_team: u8,
        matchups: &mut Vec<[usize; 2]>,
        prev_schedule_map: &HashMap<usize, TeamScheduleData>,
        attempt: u8
    ) -> Vec<TeamScheduleData> {
// -----------------------------------------------------------
        let mut schedule_data: Vec<TeamScheduleData> = TeamScheduleData::generate(&self.teams);
        let mut completed_schedule_data: Vec<TeamScheduleData> = Vec::new();
        let mut created_matchups: Vec<[usize; 2]> = Vec::new();

        let mut rng: ThreadRng = rng();
        while schedule_data.len() > 0 {
            if !Self::generate_irregular_match(&mut schedule_data, prev_schedule_map, &mut rng, &mut created_matchups, &mut completed_schedule_data, matches_per_team) {
                return Vec::new();
            }
        }

        // Add created_matchups to matchups here.
        matchups.append(&mut created_matchups);
        return completed_schedule_data;
    }

    // Generate a single irregular match.
    // Return whether successful or not.
    fn generate_irregular_match(
        schedule_data: &mut Vec<TeamScheduleData>,
        prev_schedule_map: &HashMap<usize, TeamScheduleData>,
        rng: &mut ThreadRng,
        created_matchups: &mut Vec<[usize; 2]>,
        completed_schedule_data: &mut Vec<TeamScheduleData>,
        matches_per_team: u8
    ) -> bool {
// ---------------------------------------------------------------------
        // Randomise and sort.
        schedule_data.shuffle(rng);
        TeamScheduleData::sort_default_random(schedule_data, prev_schedule_map, rng);
        let mut temp_schedule_data: Vec<TeamScheduleData> = schedule_data.clone();

        let mut team1: TeamScheduleData = temp_schedule_data[0].clone();
        temp_schedule_data.remove(0);

        // Remove every item in temp_schedule_data that already plays against team1.
        let blacklist: HashSet<usize> = team1.get_all_opponents();
        temp_schedule_data.retain(|data: &TeamScheduleData| !blacklist.contains(&data.team_id));

        if temp_schedule_data.len() == 0 {
            return false;
        }

        let home_filter: Vec<TeamScheduleData> = TeamScheduleData::filter_for_home_game(&temp_schedule_data, &prev_schedule_map, matches_per_team);
        let away_filter: Vec<TeamScheduleData> = TeamScheduleData::filter_for_away_game(&temp_schedule_data, &prev_schedule_map, matches_per_team);

        if home_filter.len() == 0 && away_filter.len() == 0 {
            return false;
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
            TeamScheduleData::sort_away_random(&mut temp_schedule_data, &prev_schedule_map, rng);
            team2 = temp_schedule_data[0].clone();
            created_matchups.push([team1.team_id, team2.team_id]);

            team1.home_matches.insert(team2.team_id);
            team2.away_matches.insert(team1.team_id);
        }

        // team1 needs an away game.
        else {
            temp_schedule_data = away_filter;
            TeamScheduleData::sort_home_random(&mut temp_schedule_data, &prev_schedule_map, rng);
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

        // Move the teams away that cannot have any more matches.
        TeamScheduleData::move_completed(schedule_data, completed_schedule_data, matches_per_team);

        // Success!
        return true;
    }

    // Attempt to generate an irregular schedule of matches.
    // Return team schedule datas if successful. Otherwise return an empty vector.
    fn attempt_irregular_generation(
        &mut self, matches_per_team: u8,
        matchups: &mut Vec<[usize; 2]>,
        prev_schedule_data: Vec<TeamScheduleData>,
        attempts: u8,
    ) -> Vec<TeamScheduleData> {
        let prev_schedule_map: HashMap<usize, TeamScheduleData> = TeamScheduleData::vector_to_hashmap(prev_schedule_data);
        for i in (Range {start: 0, end: attempts}) {
            let data: Vec<TeamScheduleData> = self.generate_irregular_matches(matches_per_team, matchups, &prev_schedule_map, i);
            if data.len() > 0 { return data; }
            self.failures += 1;
        }

        return Vec::new();
    }
}