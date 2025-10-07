// Methods for generating match schedules for Stage.
use std::collections::{HashMap, HashSet};
use std::ops::Range;
use rand::{rng, rngs::ThreadRng};
use rand::seq::SliceRandom;

use crate::types::TeamId;
use crate::competition::stage::{Stage, TeamData, rules};
use super::sorting;

#[derive(Default, Clone)]
pub struct TeamScheduleData {
    pub team_id: TeamId,
    home_matches: HashSet<TeamId>,   // Contains teams that the team plays against home.
    away_matches: HashSet<TeamId>,   // Contains teams that the team plays against away.
}

// Methods
impl TeamScheduleData {
    pub fn get_home_match_count(&self, prev: &Self) -> u8 {
        match (self.home_matches.len() + prev.home_matches.len()).try_into() {
            Ok(n) => n,
            Err(e) => panic!("{e}")
        }
    }

    pub fn get_away_match_count(&self, prev: &Self) -> u8 {
        match (self.away_matches.len() + prev.away_matches.len()).try_into() {
            Ok(n) => n,
            Err(e) => panic!("{e}")
        }
    }

    // Add home and away matches together.
    pub fn get_match_count(&self, prev: &Self) -> u8 {
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
    fn get_all_opponents(&self) -> HashSet<TeamId> {
        // Using bitwise-or to merge the HashSets.
        &self.home_matches | &self.away_matches
    }

    // Get the difference between home and away matches.
    // Positive values indicate there are more home matches.
    // Negative values indicate there are more away matches.
    pub fn get_home_away_difference(&self, prev: &Self) -> i8 {
        let home_matches: i8 = match self.get_home_match_count(prev).try_into() {
            Ok(n) => n,
            Err(e) => panic!("{e}")
        };
        let away_matches: i8 = match self.get_away_match_count(prev).try_into() {
            Ok(n) => n,
            Err(e) => panic!("{e}")
        };

        return home_matches - away_matches;
    }

    // Check that the team has enough matches, and that home and away matches are balanced.
    fn is_valid_schedule(&self, matches: u8) -> bool {
        let home_count: i8 = match self.get_home_match_count(&Self::default()).try_into() {
            Ok(n) => n,
            Err(e) => panic!("{e}")
        };
        let away_count: i8 = match self.get_away_match_count(&Self::default()).try_into() {
            Ok(n) => n,
            Err(e) => panic!("{e}")
        };
        let total_count: u8 = (home_count + away_count) as u8;

        total_count == matches &&
        (home_count - away_count).abs() <= 1
    }

    // Check if the schedule data is full (no more matches can be inserted).
    fn is_full(&self, matches: u8) -> bool {
        self.get_match_count(&Self::default()) >= matches
    }

    fn get_debug_info(&self, prev: &Self) -> String {
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

    // Get a new schedule_data vector with only teams that can play away games.
    fn filter_for_home_game(schedule_data: &Vec<Self>, prev_schedule_map: &HashMap<TeamId, Self>, matches: u8) -> Vec<Self> {
        let mut filtered: Vec<Self> = Vec::new();
        for item in schedule_data {
            let prev: &Self = match prev_schedule_map.get(&item.team_id) {
                Some(prev_ref) => prev_ref,
                None => &Self::default(),
            };

            if item.can_have_away_games(prev, matches) {
                filtered.push(item.clone())
            };
        }
        return filtered;
    }

    // Get a new schedule_data vector with only teams that can play home games.
    fn filter_for_away_game(schedule_data: &Vec<Self>, prev_schedule_map: &HashMap<TeamId, Self>, matches: u8) -> Vec<Self> {
        let mut filtered: Vec<Self> = Vec::new();
        for item in schedule_data {
            let prev: &Self = match prev_schedule_map.get(&item.team_id) {
                Some(prev_ref) => prev_ref,
                None => &Self::default(),
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
    fn vector_to_hashmap(schedule_data: Vec<Self>) -> HashMap<TeamId, Self> {
        let mut map: HashMap<TeamId, Self> = HashMap::new();
        for item in schedule_data {
            map.insert(item.team_id, item);
        }
        return map;
    }

    fn get_debug_info_all(schedule_data: &Vec<Self>, prev_schedule_map: &HashMap<TeamId, Self>) {
        let mut log: String = String::new();
        for team in schedule_data {
            let prev: &Self = match prev_schedule_map.get(&team.team_id) {
                Some(p) => p,
                None => &Self::default(),
            };

            log += &team.get_debug_info(prev);
            log += "\n\n";
        }

        println!("{}", log.trim_end_matches("\n").to_string());
    }
}

// Functional
impl Stage {
    // Generate matches where every team plays every other home and away.
    fn generate_full_round(&self, matchups: &mut Vec<[TeamId; 2]>) {
        let team_ids: Vec<TeamId> = self.get_team_ids();

        for home_id in team_ids.iter() {
            for away_id in team_ids.iter() {
                if home_id != away_id { matchups.push([*home_id, *away_id]) }
            }
        }
    }

    // Generate matches for a round robin stage.
    pub fn generate_round_robin_matches(&mut self) -> Vec<[TeamId; 2]> {
        // How many times should uncertain generations be attempted before giving up.
        const ATTEMPTS: u8 = u8::MAX;

        let matches_in_round: u8 = self.get_round_length();
        let matches_in_full_round: u8 = matches_in_round * 2;
        let mut matches: u8 = self.get_theoretical_matches_per_team();
        let mut matchups: Vec<[TeamId; 2]> = Vec::new();

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
    fn generate_irregular_matches(&self, matches_per_team: u8, matchups: &mut Vec<[TeamId; 2]>, prev_schedule_map: &HashMap<TeamId, TeamScheduleData>) -> Vec<TeamScheduleData> {
        let mut schedule_data: Vec<TeamScheduleData> = TeamScheduleData::generate(&self.teams);
        let mut completed_schedule_data: Vec<TeamScheduleData> = Vec::new();
        let mut created_matchups: Vec<[TeamId; 2]> = Vec::new();

        let mut rng: ThreadRng = rng();
        while schedule_data.len() > 0 {
            if !self.generate_irregular_match(&mut schedule_data, prev_schedule_map, &mut rng, &mut created_matchups, &mut completed_schedule_data, matches_per_team) {
                return Vec::new();
            }
        }

        // Add created_matchups to matchups here.
        matchups.append(&mut created_matchups);
        return completed_schedule_data;
    }

    // Generate a single irregular match. Return whether successful or not.
    fn generate_irregular_match(&self, schedule_data: &mut Vec<TeamScheduleData>, prev_schedule_map: &HashMap<TeamId, TeamScheduleData>,
    rng: &mut ThreadRng, created_matchups: &mut Vec<[TeamId; 2]>, completed_schedule_data: &mut Vec<TeamScheduleData>, matches_per_team: u8
    ) -> bool {
        // Randomise and sort.
        schedule_data.shuffle(rng);
        sorting::sort_default(&rules::RoundRobin::MATCH_GEN_TYPE, schedule_data, prev_schedule_map, rng);
        let mut temp_schedule_data: Vec<TeamScheduleData> = schedule_data.clone();

        let mut team1: TeamScheduleData = temp_schedule_data[0].clone();
        temp_schedule_data.remove(0);

        // Remove every item in temp_schedule_data that already plays against team1.
        let blacklist: HashSet<TeamId> = team1.get_all_opponents();
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

        let home_away_diff: i8 = team1.get_home_away_difference(prev_team1);
        let mut team2: TeamScheduleData;

        // team1 needs a home game.
        if away_filter.len() == 0 || (home_filter.len() > 0 && home_away_diff <= 0) {
            temp_schedule_data = home_filter;
            sorting::sort_away(&rules::RoundRobin::MATCH_GEN_TYPE, &mut temp_schedule_data, prev_schedule_map, rng);
            team2 = temp_schedule_data[0].clone();
            created_matchups.push([team1.team_id, team2.team_id]);

            team1.home_matches.insert(team2.team_id);
            team2.away_matches.insert(team1.team_id);
        }

        // team1 needs an away game.
        else {
            temp_schedule_data = away_filter;
            sorting::sort_home(&rules::RoundRobin::MATCH_GEN_TYPE, &mut temp_schedule_data, prev_schedule_map, rng);
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
        matchups: &mut Vec<[TeamId; 2]>,
        prev_schedule_data: Vec<TeamScheduleData>,
        attempts: u8,
    ) -> Vec<TeamScheduleData> {
        let prev_schedule_map: HashMap<TeamId, TeamScheduleData> = TeamScheduleData::vector_to_hashmap(prev_schedule_data);

        let team1_sort: rules::MatchGenType = rules::RoundRobin::MATCH_GEN_TYPE.clone();
        let team2_sort: rules::MatchGenType = rules::RoundRobin::MATCH_GEN_TYPE.clone();

        let team1_sorts: Vec<rules::MatchGenType> = if team1_sort == rules::MatchGenType::Alternating {
           Vec::from([rules::MatchGenType::Random, rules::MatchGenType::MatchCount])
        }
        else {
            Vec::from([team1_sort.clone()])
        };
        let team2_sorts: Vec<rules::MatchGenType> = if team2_sort == rules::MatchGenType::Alternating {
            Vec::from([rules::MatchGenType::MatchCount, rules::MatchGenType::Random])
        }
        else {
            Vec::from([team2_sort.clone()])
        };

        let mut data: Vec<TeamScheduleData> = Vec::new();
        for i in (Range {start: 0, end: attempts}) {
            // Alternate between sort_types.
            let index: usize = i as usize;
            let (team1_index, team2_index) = if team1_sorts.len() > 1 && team2_sorts.len() > 1 {
                (index / team1_sorts.len() % team1_sorts.len(), index % team2_sorts.len())
            }
            else {
                (index % team1_sorts.len(), index % team2_sorts.len())
            };

            // self.round_robin_rules.sort_team1 = team1_sorts[team1_index].clone();
            // self.round_robin_rules.sort_team2 = team2_sorts[team2_index].clone();

            data = self.generate_irregular_matches(matches_per_team, matchups, &prev_schedule_map);
            if data.len() > 0 {
                break;
            }
            self.failures += 1;
        }

        // Give MatchGenType::Alternating back to the round_robin_rules so it can be used next time.
        // self.round_robin_rules.sort_team1 = team1_sort;
        // self.round_robin_rules.sort_team2 = team2_sort;
        return data;
    }
}