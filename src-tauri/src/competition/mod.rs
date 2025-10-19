// A competition can be its own, contained thing, a part of a bigger competition, or both.
pub mod season;
pub mod format;
pub mod knockout_generator;

use std::{cmp::Ordering, iter::zip};

use serde_json::json;

use crate::{competition::season::{ranking::{get_sort_functions, RankCriteria}, team::TeamCompData, Season}, database::{COMPETITIONS, SEASONS}, team::Team, time::{db_string_to_date, AnnualWindow}, types::{convert, CompetitionId, TeamId}};

use self::format::Format;

#[derive(Debug, serde::Serialize)]
#[derive(Default, Clone)]
pub struct Competition {
    pub id: CompetitionId,
    pub name: String,
    pub season_window: AnnualWindow,  // Dates between which this competition is played.
    connections: Vec<CompConnection>,
    min_no_of_teams: u8,
    pub format: Option<Format>,
    rank_criteria: Vec<RankCriteria>,

    pub child_comp_ids: Vec<CompetitionId>,
    pub parent_comp_id: CompetitionId,
    pub is_tournament_tree: bool,  // Make this true if the competition consists solely of child knockout rounds.
}

// Basics.
impl Competition {
    // Create a new ID.
    fn create_id(&mut self, id: usize) {
        self.id = match id.try_into() {
            Ok(n) => n,
            Err(e) => panic!("{e}")
        };
    }

    // Build a Competition element.
    fn build(name: &str, teams: &[Team], season_window: AnnualWindow, connections: Vec<CompConnection>,
    min_no_of_teams: u8, format: Option<Format>, rank_criteria: Vec<RankCriteria>, child_comp_ids: Vec<CompetitionId>) -> Self {
        let mut comp = Self::default();
        comp.name = name.to_string();
        comp.season_window = season_window;
        comp.connections = connections;

        // If min_no_of_teams is 0, competition must have teams assigned to it.
        comp.min_no_of_teams = match min_no_of_teams {
            0 => convert::usize_to_u8(teams.len()),
            _ => min_no_of_teams
        };

        comp.format = format;
        comp.rank_criteria = rank_criteria;

        comp.child_comp_ids = child_comp_ids;

        return comp;
    }

    // Build a competition element and save it to the database.
    pub fn build_and_save(name: &str, mut teams: Vec<Team>, season_window: AnnualWindow, connections: Vec<CompConnection>,
    min_no_of_teams: u8, format: Option<Format>, rank_criteria: Vec<RankCriteria>, child_comp_ids: Vec<CompetitionId>) -> Self {
        let mut comp = Self::build(name, &teams, season_window, connections, min_no_of_teams, format, rank_criteria, child_comp_ids);

        comp.save_new(&teams);

        for team in teams.iter_mut() {
            team.primary_comp_id = comp.id;
            team.save();
        }

        return comp;
    }

    pub fn fetch_from_db(id: &CompetitionId) -> Self {
        COMPETITIONS.lock().unwrap().get(id).cloned().unwrap()
    }

    // Save a competition to the database for the first time.
    fn save_new(&mut self, teams: &[Team]) {
        self.create_id(COMPETITIONS.lock().unwrap().len() + 1);
        self.save();

        // Let's create a seasons entry for this competition so we never have to check for its existence.
        SEASONS.lock().unwrap().insert(self.id, Vec::new());

        // Create and save the first season.
        let team_ids: Vec<u8> = teams.iter().map(|a| a.id).collect();
        self.create_new_season(&team_ids);
    }

    // Update the Competition to database.
    pub fn save(&self) {
        COMPETITIONS.lock()
            .expect(&format!("something went wrong when trying to update Competition {}: {} to COMPETITIONS", self.id, self.name))
            .insert(self.id, self.clone());
    }

    // Create a new season for the competition.
    fn create_new_season(&self, teams: &[TeamId]) {
        Season::build_and_save(self, teams);
    }

    // Create a new season for this competition and all its child competitions.
    fn create_new_seasons(&self, teams: &[TeamId]) {
        self.create_new_season(teams);

        // Saves an unnecessary element creation.
        if self.child_comp_ids.is_empty() { return; }

        let child_teams = Vec::new();
        for id in self.child_comp_ids.iter() {
            Competition::fetch_from_db(id).create_new_seasons(&child_teams);        }
    }

    // Create new season for this competition and its child competitions.
    pub fn create_and_setup_seasons(&self, teams: &[TeamId]) {
        self.create_new_seasons(teams);
        self.setup_season(&mut Vec::new());
    }

    // Give child competitions this competition's ID.
    pub fn give_id_to_children_comps(&self) {
        for id in self.child_comp_ids.iter() {
            let mut child_comp = Competition::fetch_from_db(id);
            child_comp.parent_comp_id = self.id;
            child_comp.save();
        }
    }

    // Get the name of this competition with all parent competition names.
    fn get_full_name(&self, string: &str) -> String {
        let mut name = if string.is_empty() {
            self.name.clone()
        } else {
            format!("{} {}", self.name, string)
        };

        if self.parent_comp_id != 0 {
            name = Competition::fetch_from_db(&self.parent_comp_id).get_full_name(&name);
        }

        return name;
    }

    // Get the amount of seasons this competition has stored.
    pub fn get_seasons_amount(&self) -> usize {
        SEASONS.lock().unwrap().get(&self.id).expect(&format!("{}: {} has no seasons", self.id, self.name)).len()
    }

    // Get the round robin format, if competition has one.
    pub fn get_round_robin_format(&self) -> Option<format::round_robin::RoundRobin> {
        if self.format.is_none() {
            None
        } else {
            self.format.as_ref().unwrap().round_robin.clone()
        }
    }

    // Get the current season of the competition.
    fn get_current_season(&self) -> Season {
        Season::fetch_from_db(&self.id, self.get_seasons_amount() - 1)
    }

    // Get the teams in the competition's current season.
    pub fn get_teams(&self) -> Vec<Team> {
        self.get_current_season().get_teams()
    }
}

// Functional.
impl Competition {
    // Set up a season that has already been created and saved to the database.
    pub fn setup_season(&self, teams: &mut Vec<TeamCompData>) {
        let mut season = Season::fetch_from_db(&self.id, self.get_seasons_amount() - 1);

        while !teams.is_empty() && !season.has_enough_teams(self.min_no_of_teams) {
            season.teams.push(teams.swap_remove(teams.len() - 1));
        }

        if season.has_enough_teams(self.min_no_of_teams) {
            season.setup(self);
        }

        season.save();
    }

    // Sort a given list of teams with the competition's sort criteria.
    fn sort_some_teams(&self, teams: &mut Vec<TeamCompData>) {
        let sort_functions = get_sort_functions();
        let rr = self.get_round_robin_format();

        teams.sort_by(|a, b| {
            let mut order = Ordering::Equal;
            for criterium in self.rank_criteria.iter() {
                order = sort_functions[&criterium](a, b, &rr);

                if order.is_ne() { break; }
            }
            order
        });
    }

    // Get relevant information for a competition screen.
    pub fn get_comp_screen_json(&self) -> serde_json::Value {
        let season = Season::fetch_from_db(&self.id, self.get_seasons_amount() - 1);

        json!({
            "name": self.name,
            "full_name": self.get_full_name(""),
            "format": if self.format.is_none() {
                serde_json::Value::Null
            } else {
                self.format.as_ref().unwrap().get_comp_screen_json()
            },
            "season": season.get_comp_screen_json(self),
            "child_comp_ids": self.child_comp_ids,
            "parent_comp_id": self.parent_comp_id,
            "is_tournament_tree": self.is_tournament_tree
        })
    }

    // Get relevant information for a tournament tree competition screen.
    pub fn get_tournament_comp_screen_json(&self) -> serde_json::Value {
        let mut child_comps: Vec<Competition> = self.child_comp_ids.iter().map(|id| Competition::fetch_from_db(id)).collect();
        let season_index = self.get_seasons_amount() - 1;
        let mut child_seasons: Vec<Season> = child_comps.iter().map(|a| Season::fetch_from_db(&a.id, season_index)).collect();

        let mut upcoming_games = Vec::new();
        let mut played_games = Vec::new();
        let mut rounds = Vec::new();
        for (season, comp) in zip(child_seasons.iter_mut(), child_comps.iter_mut()) {
            upcoming_games.append(&mut season.upcoming_games);
            played_games.append(&mut season.played_games);

            let mut round = season.knockout_round.as_ref().unwrap().get_comp_screen_json();
            round["name"] = json!(comp.name);
            rounds.push(round);
        }

        // Upcoming games with next last.
        upcoming_games.sort_by(|a, b| db_string_to_date(&b.date).cmp(&db_string_to_date(&a.date)));
        // Played games with most recent last.
        played_games.sort_by(|a, b| db_string_to_date(&a.date).cmp(&db_string_to_date(&b.date)));

        let mut comp_json = self.get_comp_screen_json();
        comp_json["season"]["upcoming_games"] = upcoming_games.iter().map(|a| a.get_comp_screen_json()).collect();
        comp_json["season"]["played_games"] = played_games.iter().map(|a| a.get_comp_screen_json()).collect();
        comp_json["season"]["rounds"] = json!(rounds);

        return comp_json;
    }
}

// What to do with the seed of the team.
#[derive(Debug, serde::Serialize)]
#[derive(Clone)]
pub enum Seed {
    // Get the seed from the team's final standing in the previous competition.
    GetFromPosition,

    // Preserve the team's seed from the previous competition.
    Preserve,
}

// Stores data for which teams to go to which competition.
#[derive(Debug, serde::Serialize)]
#[derive(Clone)]
pub struct CompConnection {
    teams_from_positions: [u8; 2],
    comp_to_connect: CompetitionId,
    team_seeds: Seed,
    stats_carry_over: bool,
}

impl CompConnection {
    // Build the element.
    pub fn build(teams_from_positions: [u8; 2], comp_to_connect: CompetitionId, team_seeds: Seed, stats_carry_over: bool) -> Self {
        Self {
            teams_from_positions: teams_from_positions,
            comp_to_connect: comp_to_connect,
            team_seeds: team_seeds,
            stats_carry_over: stats_carry_over
        }
    }

    // Send teams onwards to the next stage.
    fn send_teams(&self, teams: &[TeamCompData]) {
        let mut teamdata = Vec::new();

        for i in self.teams_from_positions[0] - 1..self.teams_from_positions[1]  {
            let seed = match self.team_seeds {
                Seed::GetFromPosition => i + 1,
                Seed::Preserve => teams[i as usize].seed,
            };

            let team = if self.stats_carry_over {
                let mut t = teams[i as usize].clone();
                t.seed = seed;
                t
            } else {
                TeamCompData::build(teams[i as usize].team_id, seed)
            };

            teamdata.push(team);
        }

        Competition::fetch_from_db(&self.comp_to_connect).setup_season(&mut teamdata);
    }
}