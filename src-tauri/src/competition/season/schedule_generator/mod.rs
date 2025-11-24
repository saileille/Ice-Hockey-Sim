// Scheduling-related methods that are valid for both Knockout and RoundRobin.

mod sorting;

use std::{collections::{HashMap, HashSet}, iter::zip};
use rand::{rngs::ThreadRng, seq::SliceRandom, Rng};
use ::time::Date;

use crate::{
    competition::{Competition, format::round_robin::RoundRobin as RoundRobinFormat, season::{Season, knockout_round::{KnockoutPair, KnockoutRound as KnockoutRoundSeason}, team::TeamSeason}}, match_event::Game, time::get_dates, types::{Db, TeamId, convert}
};

impl Season {
    // Generate a match schedule for round robin stages.
    pub async fn generate_schedule(&mut self, db: &Db, comp: &Competition) {
        let mut match_pool = self.generate_match_pool(db, comp).await;
        let matchdays = generate_matchdays(&mut match_pool);
        assign_dates(db, matchdays, self.start_date, self.end_date, comp, true).await;
    }

    // Generate matches for a round robin stage.
    async fn generate_match_pool(&self, db: &Db, comp: &Competition) -> Vec<[TeamId; 2]> {
        let teams = comp.current_season_teamdata(db).await;

        // How many times should uncertain generations be attempted before giving up.
        const ATTEMPTS: u8 = u8::MAX;
        let round_robin = comp.format.as_ref().unwrap().round_robin.as_ref().unwrap();

        let matches_in_round = round_robin.get_round_length(db, self).await;
        let matches_in_full_round = matches_in_round * 2;
        let mut matches = round_robin.get_theoretical_matches_per_team(db, self).await;
        let mut match_pool = Vec::new();

        // Complete rounds.
        while matches >= matches_in_full_round {
            self.generate_full_round(&mut match_pool, &teams);
            matches -= matches_in_full_round;
        }

        // Half rounds.
        let mut rng = rand::rng();
        let mut prev_schedule_data = Vec::new();
        if matches >= matches_in_round {
            prev_schedule_data = self.attempt_irregular_generation(&mut rng, matches_in_round, &mut match_pool, prev_schedule_data, ATTEMPTS, &teams);

            // If unsuccessful, move on to the next part with one match less.
            if prev_schedule_data.len() == 0 {
                matches = matches_in_round - 1;

                // Making sure we are not trying the impossible.
                if teams.len() % 2 != 0 && matches % 2 != 0 {
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
            prev_schedule_data = self.attempt_irregular_generation(&mut rng, matches, &mut match_pool, prev_schedule_data, ATTEMPTS, &teams);

            // If unsuccessful, try again with one match less.
            if prev_schedule_data.len() == 0 {
                matches -= 1;
            }

            // Otherwise, exit the loop and then the method.
            else {
                break;
            }

            // Making sure we are not trying the impossible.
            if teams.len() % 2 != 0 && matches % 2 != 0 {
                matches -= 1;
            }
        }

        return match_pool;
    }

    // Generate matches where every team plays every other home and away.
    fn generate_full_round(&self, match_pool: &mut Vec<[TeamId; 2]>, teams: &[TeamSeason]) {
        let team_ids: Vec<TeamId> = teams.iter().map(|a| a.team_id).collect();

        for home_id in team_ids.iter() {
            for away_id in team_ids.iter() {
                if home_id != away_id { match_pool.push([*home_id, *away_id]) }
            }
        }
    }

    // Attempt to generate an irregular schedule of matches.
    // Return team schedule datas if successful. Otherwise return an empty vector.
    fn attempt_irregular_generation(
        &self, rng: &mut ThreadRng, matches_per_team: u8,
        match_pool: &mut Vec<[TeamId; 2]>,
        prev_schedule_data: Vec<TeamScheduleData>,
        attempts: u8, teams: &[TeamSeason]
    ) -> Vec<TeamScheduleData> {
        let prev_schedule_map = TeamScheduleData::vector_to_hashmap(prev_schedule_data);

        let mut data = Vec::new();
        for _ in 0..attempts {
            // Alternate between sort_types.

            // self.round_robin_rules.sort_team1 = team1_sorts[team1_index].clone();
            // self.round_robin_rules.sort_team2 = team2_sorts[team2_index].clone();

            data = self.generate_irregular_matches(rng, matches_per_team, match_pool, &prev_schedule_map, teams);
            if data.len() > 0 {
                break;
            }
            // self.failures += 1;
        }

        // Give MatchGenType::Alternating back to the round_robin_rules so it can be used next time.
        // self.round_robin_rules.sort_team1 = team1_sort;
        // self.round_robin_rules.sort_team2 = team2_sort;
        return data;
    }

    // Generate a match schedule with arbitrary number of games.
    // Add to an existing match pool vector if successful.
    // Return the schedule data. If unsuccessful, return empty vector.
    fn generate_irregular_matches(&self, rng: &mut ThreadRng, matches_per_team: u8, match_pool: &mut Vec<[TeamId; 2]>, prev_schedule_map: &HashMap<TeamId, TeamScheduleData>, teams: &[TeamSeason]) -> Vec<TeamScheduleData> {
        let mut schedule_data = TeamScheduleData::generate(teams);
        let mut completed_schedule_data = Vec::new();
        let mut created_matches = Vec::new();

        while schedule_data.len() > 0 {
            if !self.generate_irregular_match(rng, &mut schedule_data, prev_schedule_map, &mut created_matches, &mut completed_schedule_data, matches_per_team) {
                return Vec::new();
            }
        }

        // Add created matches to match pool here.
        match_pool.append(&mut created_matches);
        return completed_schedule_data;
    }

    // Generate a single irregular match. Return whether successful or not.
    fn generate_irregular_match(&self, rng: &mut ThreadRng, schedule_data: &mut Vec<TeamScheduleData>, prev_schedule_map: &HashMap<TeamId, TeamScheduleData>,
    created_matches: &mut Vec<[TeamId; 2]>, completed_schedule_data: &mut Vec<TeamScheduleData>, matches_per_team: u8
    ) -> bool {
        // Randomise and sort.
        schedule_data.shuffle(rng);
        sorting::sort_default(rng, &RoundRobinFormat::MATCH_GEN_TYPE, schedule_data, prev_schedule_map);
        let mut temp_schedule_data = schedule_data.clone();

        let mut team1 = temp_schedule_data.swap_remove(0);

        // Remove every item in temp_schedule_data that already plays against team1.
        let opponents = team1.get_all_opponents();
        temp_schedule_data.retain(|team: &TeamScheduleData| !opponents.contains(&team.team_id));

        if temp_schedule_data.is_empty() {
            return false;
        }

        let home_filter = TeamScheduleData::filter_for_home_game(&temp_schedule_data, &prev_schedule_map, matches_per_team);
        let away_filter = TeamScheduleData::filter_for_away_game(&temp_schedule_data, &prev_schedule_map, matches_per_team);

        if home_filter.is_empty() && away_filter.is_empty() {
            return false;
        }

        // Get the match data from a match generation that occurred previously.
        let prev_team1 = match prev_schedule_map.get(&team1.team_id) {
            Some(p) => p,
            None => &TeamScheduleData::default(),
        };

        let home_away_diff = team1.get_home_away_difference(prev_team1);
        let mut team2;

        // team1 needs a home game.
        if away_filter.len() == 0 || (home_filter.len() > 0 && home_away_diff <= 0) {
            temp_schedule_data = home_filter;
            sorting::sort_away(rng, &RoundRobinFormat::MATCH_GEN_TYPE, &mut temp_schedule_data, prev_schedule_map);
            team2 = temp_schedule_data.swap_remove(0);
            created_matches.push([team1.team_id, team2.team_id]);

            team1.home_matches.push(team2.team_id);
            team2.away_matches.push(team1.team_id);
        }

        // team1 needs an away game.
        else {
            temp_schedule_data = away_filter;
            sorting::sort_home(rng, &RoundRobinFormat::MATCH_GEN_TYPE, &mut temp_schedule_data, prev_schedule_map);
            team2 = temp_schedule_data.swap_remove(0);
            created_matches.push([team2.team_id, team1.team_id]);

            team1.away_matches.push(team2.team_id);
            team2.home_matches.push(team1.team_id);
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
}

// Give each matchday a date, build the games and return them.
pub async fn assign_dates(db: &Db, matchdays: Vec<Vec<[TeamId; 2]>>, start_date: Date, end_date: Date, comp: &Competition, randomise_order: bool) {
    let mut dates = get_dates(start_date, end_date);
    let mut game_dates = Vec::new();

    for _ in 0..matchdays.len() {
        game_dates.push(dates.swap_remove(rand::random_range(0..dates.len())));
    }

    if !randomise_order { game_dates.sort(); }

    for (date, matchday) in zip(game_dates.iter(), matchdays.iter()) {
        build_games(db, matchday, *date, comp).await;
    }
}

// Convert the simple representations of two teams into Game elements.
async fn build_games(db: &Db, match_pool: &[[TeamId; 2]], date: Date, comp: &Competition) {
    let season_id = comp.current_season_id(db).await;
    for matchup in match_pool {
        Game::build_and_save(db, matchup[0], matchup[1], season_id, date).await;
    }
}

// Generate a single matchday.
// Attempts to make as many teams as possible to play at the same time.
fn generate_matchday(rng: &mut ThreadRng, match_pool: &mut Vec<[TeamId; 2]>) -> Vec<[TeamId; 2]> {
    let mut valid_matches = match_pool.clone();
    let mut matchday = Vec::new();

    while !valid_matches.is_empty() {
        let game = valid_matches.swap_remove(rng.random_range(0..valid_matches.len()));
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
    let mut rng = rand::rng();
    let mut matchdays = Vec::new();
    while match_pool.len() > 0 {
        matchdays.push(generate_matchday(&mut rng, match_pool));
    }
    return matchdays;
}

impl KnockoutRoundSeason {
    // Generate matchdays for the knockout round.
    pub fn generate_matchdays(&self, comp: &Competition) -> Vec<Vec<[TeamId; 2]>> {
        let matches = comp.format.as_ref().unwrap().knockout_round.as_ref().unwrap().get_maximum_matches_in_pair();
        let mut matchdays = vec![Vec::new(); matches as usize];
        for pair in self.pairs.iter() {
            pair.generate_matchdays(&mut matchdays);
        }

        return matchdays;
    }
}

impl KnockoutPair {
    // Generate matchdays for the knockout pair.
    fn generate_matchdays(&self, matchdays: &mut Vec<Vec<[TeamId; 2]>>) {
        for (i, day) in matchdays.iter_mut().enumerate() {
            if i % 2 == 0 {
                day.push([self.home.team_id, self.away.team_id]);
            }
            else {
                day.push([self.away.team_id, self.home.team_id]);
            }
        }
    }
}

#[derive(Default, Clone)]
pub struct TeamScheduleData {
    pub team_id: TeamId,
    home_matches: Vec<TeamId>,   // Contains teams that the team plays against home.
    away_matches: Vec<TeamId>,   // Contains teams that the team plays against away.
}

// Methods
impl TeamScheduleData {
    pub fn get_home_match_count(&self, prev: &Self) -> u8 {
        convert::int::<usize, u8>(self.home_matches.len() + prev.home_matches.len())
    }

    pub fn get_away_match_count(&self, prev: &Self) -> u8 {
        convert::int::<usize, u8>(self.away_matches.len() + prev.away_matches.len())
    }

    // Add home and away matches together.
    pub fn get_match_count(&self, prev: &Self) -> u8 {
        self.get_home_match_count(prev) + self.get_away_match_count(prev)
    }

    // Check if the team can have any more home games.
    fn can_have_home_games(&self, prev: &Self, matches: u8) -> bool {
        let total_matches = matches + prev.get_match_count( &Self::default());
        self.get_home_match_count(prev) < (total_matches + 1) / 2
    }

    // Check if the team can have any more away games.
    fn can_have_away_games(&self, prev: &Self, matches: u8) -> bool {
        let total_matches = matches + prev.get_match_count( &Self::default());
        self.get_away_match_count(prev) < (total_matches + 1) / 2
    }

    // Get a combined vector of home_matches and away_matches.
    fn get_all_opponents(&self) -> Vec<TeamId> {
        let mut combined = Vec::new();
        combined.append(&mut self.home_matches.clone());
        combined.append(&mut self.away_matches.clone());

        let hash_set: HashSet<TeamId> = HashSet::from_iter(combined);
        return Vec::from_iter(hash_set);
    }

    // Get the difference between home and away matches.
    // Positive values indicate there are more home matches.
    // Negative values indicate there are more away matches.
    pub fn get_home_away_difference(&self, prev: &Self) -> i8 {
        let home_matches = convert::int::<u8, i8>(self.get_home_match_count(prev));
        let away_matches = convert::int::<u8, i8>(self.get_away_match_count(prev));

        return home_matches - away_matches;
    }

    // Check if the schedule data is full (no more matches can be inserted).
    fn is_full(&self, matches: u8) -> bool {
        self.get_match_count(&Self::default()) >= matches
    }
}

// Static
impl TeamScheduleData {
    // Get a new schedule_data vector with only teams that can play away games.
    fn filter_for_home_game(schedule_data: &[Self], prev_schedule_map: &HashMap<TeamId, Self>, matches: u8) -> Vec<Self> {
        let mut filtered = Vec::new();
        for item in schedule_data {
            let prev = match prev_schedule_map.get(&item.team_id) {
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
    fn filter_for_away_game(schedule_data: &[Self], prev_schedule_map: &HashMap<TeamId, Self>, matches: u8) -> Vec<Self> {
        let mut filtered = Vec::new();
        for item in schedule_data {
            let prev = match prev_schedule_map.get(&item.team_id) {
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
        let mut index = 0;
        while index < schedule_data.len() {
            let data = &schedule_data[index];
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
    fn generate(comp_teams: &[TeamSeason]) -> Vec<Self> {
        let mut schedule_data = Vec::new();
        for team_data in comp_teams.iter() {
            schedule_data.push(Self {
                team_id: team_data.team_id,
                home_matches: Vec::new(),
                away_matches: Vec::new(),
            })
        }

        return schedule_data;
    }

    // Convert the vector of TeamScheduleData to hashmap. Consume the vector in the process.
    fn vector_to_hashmap(schedule_data: Vec<Self>) -> HashMap<TeamId, Self> {
        let mut map = HashMap::new();
        for item in schedule_data {
            map.insert(item.team_id, item);
        }
        return map;
    }
}