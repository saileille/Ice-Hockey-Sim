// A competition can be its own, contained thing, a part of a bigger competition, or both.
pub mod season;
pub mod format;
pub mod knockout_generator;

use std::{cmp::Ordering, ops::Range};

use crate::{competition::season::{ranking::{get_sort_functions, RankCriteria}, team::TeamCompData, Season}, database::{COMPETITIONS, SEASONS}, team::Team, time::AnnualWindow, types::{convert, CompetitionId, TeamId}};

use self::format::Format;

#[derive(Debug)]
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
    fn build(name: &str, teams: &Vec<Team>, season_window: AnnualWindow, connections: Vec<CompConnection>,
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
    pub fn build_and_save(name: &str, teams: &Vec<Team>, season_window: AnnualWindow, connections: Vec<CompConnection>,
    min_no_of_teams: u8, format: Option<Format>, rank_criteria: Vec<RankCriteria>, child_comp_ids: Vec<CompetitionId>) -> Self {
        let mut comp = Self::build(name, &teams, season_window, connections, min_no_of_teams, format, rank_criteria, child_comp_ids);

        comp.save_new(&teams);

        return comp;
    }

    pub fn fetch_from_db(id: &CompetitionId) -> Option<Self> {
        COMPETITIONS.lock().unwrap().get(id).cloned()
    }

    // Save a competition to the database for the first time.
    fn save_new(&mut self, teams: &Vec<Team>) {
        self.create_id(COMPETITIONS.lock().unwrap().len() + 1);
        self.save();

        // Let's create a seasons entry for this competition so we never have to check for its existence.
        SEASONS.lock().unwrap().insert(self.id, Vec::new());

        // Create and save the first season.
        let team_ids = teams.iter().map(|a| a.id).collect();
        Season::build_and_save(self, &team_ids);
    }

    // Update the Competition to database.
    pub fn save(&self) {
        COMPETITIONS.lock()
            .expect(&format!("something went wrong when trying to update Competition {}: {} to COMPETITIONS", self.id, self.name))
            .insert(self.id, self.clone());
    }

    // Give child competitions this competition's ID.
    pub fn give_id_to_children_comps(&self) {
        for id in self.child_comp_ids.iter() {
            let mut child_comp = Self::fetch_from_db(id).expect(&format!("{}: {} has child comp id of {id}", self.id, self.name));
            child_comp.parent_comp_id = self.id;
            child_comp.save();
        }
    }

    // Get the amount of seasons this competition has stored.
    pub fn get_seasons_amount(&self) -> usize {
        SEASONS.lock().unwrap().get(&self.id).expect(&format!("{}: {} has no seasons", self.id, self.name)).len()
    }

    // Get the round robin format, if competition has one.
    fn get_round_robin_format(&self) -> Option<format::round_robin::RoundRobin> {
        if self.format.is_none() {
            None
        } else {
            self.format.as_ref().unwrap().round_robin.clone()
        }
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
}

// What to do with the seed of the team.
#[derive(Debug)]
#[derive(Clone)]
pub enum Seed {
    // Get the seed from the team's final standing in the previous competition.
    GetFromPosition,

    // Preserve the team's seed from the previous competition.
    Preserve,
}

// Stores data for which teams to go to which competition.
#[derive(Debug)]
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
    fn send_teams(&self, teams: &Vec<TeamCompData>) {
        let mut teamdata = Vec::new();

        for i in (Range { start: self.teams_from_positions[0] - 1, end: self.teams_from_positions[1] })  {
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

        let comp = Competition::fetch_from_db(&self.comp_to_connect).expect(&format!("competition id {} not found", self.comp_to_connect));
        comp.setup_season(&mut teamdata);
    }
}