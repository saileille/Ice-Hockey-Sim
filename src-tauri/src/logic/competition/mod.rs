// A competition can be its own, contained thing, a part of a bigger competition, or both.
pub mod season;

pub mod comp_connection;
pub mod knockout_generator;
pub mod knockout_round;
pub mod round_robin;


use std::cmp::Ordering;

use serde::{Deserialize, Serialize};
use time::Date;

use crate::logic::{competition::{comp_connection::CompConnection, knockout_round::KnockoutRound as KnockoutRoundFormat, round_robin::RoundRobin as RoundRobinFormat, season::{Season, ranking::{RankCriteria, get_sort_functions}, team::TeamSeason}}, game, team::Team, time::AnnualWindow, types::{CompetitionId, Db, TeamId}};

#[derive(Debug, PartialEq)]
#[derive(Default, Clone, Copy, Serialize, Deserialize)]
#[derive(sqlx::Type)]
pub enum Type {
    #[default]
    Null,   // Means the competition is comprised solely of child competitions.
    RoundRobin,
    KnockoutRound,
    Tournament,
}

#[derive(Debug)]
#[derive(Default, Clone)]
pub struct Competition {
    pub id: CompetitionId,
    pub name: String,
    pub season_window: AnnualWindow,  // Dates between which this competition is played.
    pub min_no_of_teams: u8,
    pub rank_criteria: Vec<RankCriteria>,
    pub comp_type: Type,
    pub parent_id: CompetitionId,   // TODO: Some conversion between NULL in the database and 0 in the code.
}

// Basics.
impl Competition {
    // Build a Competition element.
    fn build(name: &str, no_of_teams: u8, season_window: AnnualWindow,
    min_no_of_teams: u8, rank_criteria: Vec<RankCriteria>) -> Self {
        let comp = Self {
            name: name.to_string(),
            season_window,
            rank_criteria,

            min_no_of_teams: match min_no_of_teams {
                0 => no_of_teams,
                _ => min_no_of_teams
            },

            ..Default::default()
        };

        return comp;
    }

    // Build a competition element and save it to the database.
    pub async fn build_and_save(db: &Db, today: Date, name: &str, teams: Vec<Team>, season_window: AnnualWindow, connections: Vec<CompConnection>, min_no_of_teams: u8,
    match_rules: Option<game::Rules>, round_robin_format: Option<RoundRobinFormat>, knockout_round_format: Option<KnockoutRoundFormat>, rank_criteria: Vec<RankCriteria>,
    children: Vec<Competition>) -> Self {
        let mut comp = Self::build(name, teams.len() as u8, season_window, min_no_of_teams, rank_criteria);
        comp.save_new(db, today, teams, match_rules, round_robin_format, knockout_round_format, children, connections).await;

        return comp;
    }

    // Save a competition to the database for the first time.
    async fn save_new(&mut self, db: &Db, today: Date, mut teams: Vec<Team>, match_rules: Option<game::Rules>, round_robin_format: Option<RoundRobinFormat>,
    knockout_round_format: Option<KnockoutRoundFormat>, children: Vec<Competition>, connections: Vec<CompConnection>) {
        self.save(db).await;

        match match_rules {
            Some(v) => v.save(db, self.id).await,
            None => ()
        };

        match round_robin_format {
            Some(v) => {
                self.comp_type = Type::RoundRobin;
                self.save_type(db).await;
                v.save(db, self.id).await
            },
            None => ()
        };

        match knockout_round_format {
            Some(v) => {
                self.comp_type = Type::KnockoutRound;
                self.save_type(db).await;
                v.save(db, self.id).await
            },
            None => ()
        };

        for team in teams.iter_mut() {
            team.primary_comp_id = self.id;
            team.save(db).await;
        }

        // Adding child competitions here.
        for child in children {
            child.save_parent_id(db, self.id).await;
        }

        // Adding competition connections here.
        for connection in connections {
            connection.save(db, self.id).await;
        }

        // Create and save the first season.
        let team_ids: Vec<u8> = teams.into_iter().map(|a| a.id).collect();
        self.create_new_season(db, today, &team_ids).await;
    }

    // Create a new season for the competition.
    async fn create_new_season(&self, db: &Db, today: Date, teams: &[TeamId]) {
        Season::build_and_save(db, today, self, teams).await;
    }

    // Create a new season for this competition and all its child competitions.
    async fn create_new_seasons(&self, db: &Db, today: Date, teams: &[TeamId]) {
        self.create_new_season(db, today, teams).await;
        let child_ids = self.child_ids(db).await;

        // Saves an unnecessary element creation.
        if child_ids.is_empty() { return; }

        let child_teams = Vec::new();
        for id in child_ids.into_iter() {
            Box::pin(Competition::fetch_from_db(db, id).await.create_new_seasons(db, today, &child_teams)).await;
        }
    }

    // Create new season for this competition and its child competitions.
    pub async fn create_and_setup_seasons(&self, db: &Db, today: Date, teams: &[TeamId]) {
        self.create_new_seasons(db, today, teams).await;
        self.setup_season(db, &mut Vec::new()).await;
    }

    // Get the name of this competition with all parent competition names.
    async fn full_name(&self, db: &Db) -> String {
        let mut name = self.name.clone();
        let mut o_parent = self.parent(db).await;
        while o_parent.is_some() {
            let parent = o_parent.unwrap();
            name = format!("{} {}", parent.name, name);
            o_parent = parent.parent(db).await;
        }

        return name;
    }

    // Set up a season that has already been created and saved to the database.
    pub async fn setup_season(&self, db: &Db, teams: &mut Vec<TeamSeason>) {
        let mut season = self.current_season(db).await;
        let mut no_of_teams = season.no_of_teams(db).await;

        while !teams.is_empty() && self.min_no_of_teams > no_of_teams {
            let mut team = teams.swap_remove(teams.len() - 1);
            team.season_id = season.id; // Making the team's season distinct from the parent.
            team.save(db).await;
            no_of_teams += 1;
        }

        if self.min_no_of_teams <= no_of_teams {
            Box::pin(season.setup(db, self)).await;
        }
    }

    // Sort a given list of teams with the competition's sort criteria.
    async fn sort_some_teams(&self, db: &Db, teams: &mut Vec<TeamSeason>) {
        let sort_functions = get_sort_functions();
        let rr = self.round_robin_format(db).await;

        let mut rng = rand::rng();
        teams.sort_by(|a, b| {
            let mut order = Ordering::Equal;
            for criterium in self.rank_criteria.iter() {
                order = sort_functions[&criterium](a, b, &rr, &mut rng);

                if order.is_ne() { break; }
            }
            order
        });
    }

    // Create a full competition tree.
    async fn nav_package(&self, db: &Db) -> Vec<Vec<(CompetitionId, String)>> {
        let mut select_data = Vec::new();

        let children = self.child_nav_package(db, "Overview").await;
        if children.len() > 1 { select_data.push(children); }

        let siblings = self.sibling_nav_package(db).await;
        if siblings.len() > 1 { select_data.push(siblings); }

        self.parent_nav_package(db, &mut select_data).await;

        // We need to reverse so we get the competition hierarchy from highest to lowest.
        select_data.reverse();
        return select_data;
    }

    // Get the IDs and names of all parent competitions and their siblings.
    async fn parent_nav_package(&self, db: &Db, select_data: &mut Vec<Vec<(CompetitionId, String)>>) {
        let mut o_parent = self.parent(db).await;
        while o_parent.is_some() {
            let parent = o_parent.as_ref().unwrap();

            let uncles = parent.sibling_nav_package(db).await;
            if uncles.len() > 1 { select_data.push(uncles); }

            o_parent = parent.parent(db).await;
        }
    }

    // Get the IDs of sibling competitions, including this one.
    async fn sibling_nav_package(&self, db: &Db) -> Vec<(CompetitionId, String)> {
        let mut select_data = match self.parent(db).await {
            Some(parent) => parent.child_nav_package(db, parent.name.as_str()).await,
            _ => vec![self.name_and_id()]
        };

        select_data.sort_by(|(a, _), (b, _)| a.cmp(&b));

        // Replace this competition's ID with 0, to keep track of which competition is selected.
        for comp in select_data.iter_mut() {
            if comp.0 == self.id {
                comp.0 = 0;
                break;
            }
        }

        return select_data;
    }

    // Get the IDs and names of child competitions.
    async fn child_nav_package(&self, db: &Db, self_name: &str) -> Vec<(CompetitionId, String)> {
        let mut children = self.child_navs(db).await;

        // Adding this competition so selecting a child becomes possible.
        children.insert(0, (self.id, self_name.to_string()));
        return children;
    }

    // Get the name and ID of the competition.
    fn name_and_id(&self) -> (CompetitionId, String) {
        return (self.id, self.name.clone());
    }
}

// What to do with the seed of the team.
#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize)]
#[derive(sqlx::Type)]
pub enum Seed {
    #[default]
    Null,
    // Get the seed from the team's final standing in the previous competition.
    GetFromPosition,
    // Preserve the team's seed from the previous competition.
    Preserve,
}