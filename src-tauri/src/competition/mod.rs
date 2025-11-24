// A competition can be its own, contained thing, a part of a bigger competition, or both.
pub mod season;
pub mod format;
pub mod knockout_generator;

use std::{cmp::Ordering, iter::zip};

use sqlx::FromRow;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{competition::season::{Season, ranking::{RankCriteria, get_sort_functions}, team::TeamSeason}, team::Team, time::AnnualWindow, types::{CompetitionId, Db, SeasonId, TeamId}};

use self::format::Format;

#[derive(Debug, PartialEq)]
#[derive(Default, Clone, Copy, Serialize, Deserialize)]
#[derive(sqlx::Type)]
pub enum Type {
    #[default]
    Simple, // Indicates that this is either round robin or knockout round.
    Tournament,
}

#[derive(Debug)]
#[derive(Default, Clone)]
#[derive(FromRow)]
pub struct Competition {
    pub id: CompetitionId,
    #[sqlx(rename = "comp_name")]
    pub name: String,
    pub season_window: AnnualWindow,  // Dates between which this competition is played.
    min_no_of_teams: u8,
    #[sqlx(json(nullable))]
    pub format: Option<Format>,
    #[sqlx(json)]
    rank_criteria: Vec<RankCriteria>,
    pub comp_type: Type,
    parent_id: CompetitionId,
}

// Database stuff.
impl Competition {
    async fn connections_to(&self, db: &Db) -> Vec<CompConnection> {
        sqlx::query_as(
            "SELECT * FROM CompConnection WHERE origin_id = $1"
        ).bind(self.id)
        .fetch_all(db).await.unwrap()
    }

    async fn child_ids(&self, db: &Db) -> Vec<CompetitionId> {
        sqlx::query_scalar(
            "SELECT id FROM Competition WHERE parent_id = $1"
        ).bind(self.id)
        .fetch_all(db).await.unwrap()
    }

    // Return the child competitions in the order that they are scheduled.
    // E.g. regular season should come before playoffs.
    async fn children(&self, db: &Db) -> Vec<Self> {
        sqlx::query_as(
            "SELECT * FROM Competition
            WHERE parent_id = $1
            ORDER BY id ASC"
        ).bind(self.id)
        .fetch_all(db).await.unwrap()
    }

    async fn child_current_seasons(&self, db: &Db) -> Vec<Season> {
        sqlx::query_as(
            "SELECT Season.* FROM Season
            INNER JOIN Competition ON Competition.id = Season.comp_id
            WHERE Competition.id = $1
            GROUP BY Season.comp_id
            ORDER BY Competition.id ASC"
        ).bind(self.id)
        .fetch_all(db).await.unwrap()
    }
}

// Basics.
impl Competition {
    // Build a Competition element.
    async fn build(db: &Db, name: &str, no_of_teams: u8, season_window: AnnualWindow,
    min_no_of_teams: u8, format: Option<Format>, rank_criteria: Vec<RankCriteria>) -> Self {
        let comp = Self {
            id: Self::next_id(db).await,
            name: name.to_string(),
            season_window,
            format,
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
    pub async fn build_and_save(db: &Db, name: &str, teams: Vec<Team>, season_window: AnnualWindow, connections: Vec<CompConnection>,
    min_no_of_teams: u8, format: Option<Format>, rank_criteria: Vec<RankCriteria>, children: Vec<Competition>) -> Self {
        let mut comp = Self::build(db, name, teams.len() as u8, season_window, min_no_of_teams, format, rank_criteria).await;
        comp.save_new(db, teams, children, connections).await;

        return comp;
    }

    // Get the next ID to use.
    async fn next_id(db: &Db) -> CompetitionId {
        let max: Option<CompetitionId> = sqlx::query_scalar("SELECT max(id) FROM Competition").fetch_one(db).await.unwrap();
        match max {
            Some(n) => n + 1,
            _ => 1,
        }
    }

    // Fetch a competition from the database, knowing that it exists.
    pub async fn fetch_from_db(db: &Db, id: CompetitionId) -> Self {
        sqlx::query_as(
            "SELECT * FROM Competition WHERE id = $1"
        ).bind(id)
        .fetch_one(db).await.unwrap()
    }

    // Save a competition to the database for the first time.
    async fn save_new(&mut self, db: &Db, mut teams: Vec<Team>, children: Vec<Competition>, connections: Vec<CompConnection>) {
        self.save(db).await;

        for team in teams.iter_mut() {
            team.primary_comp_id = self.id;
            team.give_id(db).await;
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
        self.create_new_season(db, &team_ids).await;
    }

    async fn save_parent_id(&self, db: &Db, parent_id: CompetitionId) {
        sqlx::query(
            "UPDATE Competition SET parent_id = $1 WHERE id = $2"
        ).bind(parent_id)
        .bind(self.id)
        .execute(db).await.unwrap();
    }

    // Save the Competition to database.
    pub async fn save(&self, db: &Db) {
        sqlx::query(
            "INSERT INTO Competition
            (id, comp_name, season_window, min_no_of_teams, format, rank_criteria, comp_type, parent_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
        ).bind(self.id)
        .bind(self.name.as_str())
        .bind(&self.season_window)
        .bind(self.min_no_of_teams)
        .bind(&self.format)
        .bind(json!(self.rank_criteria))
        .bind(self.comp_type)
        .bind(self.parent_id)
        .execute(db).await.unwrap();
    }

    // Get all competitions that do not have a parent.
    pub async fn fetch_parents(db: &Db) -> Vec<Competition> {
        sqlx::query_as(
            "SELECT * FROM Competition
            WHERE parent_id = 0"
        ).fetch_all(db).await.unwrap()
    }

    // Get the ID and name of all parent competitions.
    pub async fn fetch_parent_id_and_name(db: &Db) -> Vec<(CompetitionId, String)> {
        sqlx::query_as(
            "SELECT id, comp_name FROM Competition
            WHERE parent_id = 0
            ORDER BY id ASC"
        ).fetch_all(db).await.unwrap()
    }

    // Fetch competitions that have a format (i.e. games).
    pub async fn fetch_comps_with_games(db: &Db) -> Vec<Competition> {
        sqlx::query_as(
            "SELECT * FROM Competition WHERE format IS NOT NULL"
        ).fetch_all(db).await.unwrap()
    }

    // Get the parent of this competition.
    pub async fn parent(&self, db: &Db) -> Option<Competition> {
        sqlx::query_as(
            "SELECT * FROM Competition WHERE id = $1"
        ).bind(self.parent_id)
        .fetch_optional(db).await.unwrap()
    }

    // Create a new season for the competition.
    async fn create_new_season(&self, db: &Db, teams: &[TeamId]) {
        Season::build_and_save(db, self, teams).await;
    }

    // Create a new season for this competition and all its child competitions.
    async fn create_new_seasons(&self, db: &Db, teams: &[TeamId]) {
        self.create_new_season(db, teams).await;
        let child_ids = self.child_ids(db).await;

        // Saves an unnecessary element creation.
        if child_ids.is_empty() { return; }

        let child_teams = Vec::new();
        for id in child_ids.into_iter() {
            Box::pin(Competition::fetch_from_db(db, id).await.create_new_seasons(db, &child_teams)).await;
        }
    }

    // Create new season for this competition and its child competitions.
    pub async fn create_and_setup_seasons(&self, db: &Db, teams: &[TeamId]) {
        self.create_new_seasons(db, teams).await;
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

    // Get the round robin format, if competition has one.
    pub fn round_robin_format(&self) -> Option<format::round_robin::RoundRobin> {
        if self.format.is_none() {
            None
        } else {
            self.format.as_ref().unwrap().round_robin.clone()
        }
    }

    // Get the current season of the competition.
    pub async fn current_season(&self, db: &Db) -> Season {
        sqlx::query_as(
            "SELECT * FROM Season
            WHERE comp_id = $1
            ORDER BY id DESC
            LIMIT 1"
        ).bind(self.id)
        .fetch_one(db).await.unwrap()
    }

    // Get the current season ID.
    async fn current_season_id(&self, db: &Db) -> SeasonId {
        sqlx::query_scalar(
            "SELECT id FROM Season
            WHERE comp_id = $1
            ORDER BY id DESC
            LIMIT 1"
        ).bind(self.id)
        .fetch_one(db).await.unwrap()
    }

    // Get the teams in the competition's current season.
    pub async fn current_season_teamdata(&self, db: &Db) -> Vec<TeamSeason> {
        sqlx::query_as(
            "SELECT * FROM TeamSeason
            WHERE season_id = (
                SELECT id FROM Season
                WHERE comp_id = $1
                ORDER BY id DESC
                LIMIT 1
            )
            ORDER BY rank ASC"
        ).bind(self.id)
        .fetch_all(db).await.unwrap()
    }

    // Get the IDs and names of the current season's teams, based on competition ID.
    pub async fn current_season_team_select_data_by_id(db: &Db, id: CompetitionId) -> Vec<(TeamId, String)> {
        sqlx::query_as(
            "SELECT Team.id, Team.full_name FROM Team
            INNER JOIN TeamSeason ON
            TeamSeason.team_id = Team.id
            WHERE TeamSeason.season_id = (
                SELECT id FROM Season
                WHERE comp_id = $1
                ORDER BY id DESC
                LIMIT 1
            )
            ORDER BY Team.full_name ASC"
        ).bind(id)
        .fetch_all(db).await.unwrap()
    }
}

// Functional.
impl Competition {
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
    fn sort_some_teams(&self, teams: &mut Vec<TeamSeason>) {
        let mut rng = rand::rng();
        let sort_functions = get_sort_functions();
        let rr = self.round_robin_format();

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
    async fn get_nav_data(&self, db: &Db) -> Vec<Vec<(CompetitionId, String)>> {
        let mut select_data = Vec::new();

        let children = self.get_child_package(db, "Overview").await;
        if children.len() > 1 { select_data.push(children); }

        let siblings = self.get_sibling_package(db).await;
        if siblings.len() > 1 { select_data.push(siblings); }

        self.get_parent_package(db, &mut select_data).await;

        // We need to reverse so we get the competition hierarchy from highest to lowest.
        select_data.reverse();
        return select_data;
    }

    // Get the IDs of all parent competitions and their siblings.
    async fn get_parent_package(&self, db: &Db, select_data: &mut Vec<Vec<(CompetitionId, String)>>) {
        let mut o_parent = self.parent(db).await;
        while o_parent.is_some() {
            let parent = o_parent.as_ref().unwrap();

            let uncles = parent.get_sibling_package(db).await;
            if uncles.len() > 1 { select_data.push(uncles); }

            o_parent = parent.parent(db).await;
        }
    }

    // Get the IDs of sibling competitions, including this one.
    async fn get_sibling_package(&self, db: &Db) -> Vec<(CompetitionId, String)> {
        let mut select_data = match self.parent(db).await {
            Some(parent) => parent.get_child_package(db, &parent.name).await,
            _ => vec![self.get_name_and_id()]
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
    async fn get_child_package(&self, db: &Db, self_name: &str) -> Vec<(CompetitionId, String)> {
        let mut children: Vec<(CompetitionId, String)> = self.children(db).await.iter().map(|comp| {
            comp.get_name_and_id()
        }).collect();

        // Adding this competition so selecting a child becomes possible.
        children.insert(0, (self.id, self_name.to_string()));
        return children;
    }

    // Get the name and ID of the competition.
    fn get_name_and_id(&self) -> (CompetitionId, String) {
        return (self.id, self.name.clone());
    }

    // Get relevant information for a competition screen.
    pub async fn get_comp_screen_package(&self, db: &Db) -> serde_json::Value {
        let format = if self.format.is_none() {
            serde_json::Value::Null
        } else {
            self.format.as_ref().unwrap().comp_screen_package()
        };

        json!({
            "name": self.name,
            "full_name": self.full_name(db).await,
            "format": format,
            "season": self.current_season(db).await.comp_screen_package(db, self).await,
            "comp_nav": self.get_nav_data(db).await,
            "competition_type": self.comp_type,
        })
    }

    // Get relevant information for a tournament tree competition screen.
    pub async fn get_tournament_comp_screen_package(&self, db: &Db) -> serde_json::Value {
        let mut future_games = Vec::new();
        let mut past_games = Vec::new();
        let mut rounds = Vec::new();
        for (season, comp) in zip(self.child_current_seasons(db).await, self.children(db).await) {
            future_games.append(&mut season.today_and_future_games(db).await);
            past_games.append(&mut season.past_games(db).await);

            let mut round = season.knockout_round.as_ref().unwrap().get_comp_screen_json(db).await;
            round["name"] = json!(comp.name);
            rounds.push(round);
        }

        // Already sorted by SQL queries, no need to use Rust.
        // Upcoming games with next last.
        // future_games.sort_by(|a, b| b.date.cmp(&&a.date));
        // Played games with most recent last.
        // past_games.sort_by(|a, b| a.date.cmp(&b.date));

        // Using the default competition package as base.
        let mut comp_json = self.get_comp_screen_package(db).await;

        let mut future_games_json = Vec::new();
        for game in future_games {
            future_games_json.push(game.comp_screen_package(db).await);
        }

        let mut past_games_json = Vec::new();
        for game in past_games {
            past_games_json.push(game.comp_screen_package(db).await);
        }

        comp_json["season"]["upcoming_games"] = json!(future_games_json);
        comp_json["season"]["played_games"] = json!(past_games_json);
        comp_json["season"]["rounds"] = json!(rounds);

        return comp_json;
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

// Stores data for which teams to go to which competition.
#[derive(Debug, Default, Clone)]
#[derive(FromRow)]
pub struct CompConnection {
    origin_id: CompetitionId,
    destination_id: CompetitionId,
    highest_position: u8,
    lowest_position: u8,
    team_seeds: Seed,
    stats_carry_over: bool,
}

impl CompConnection {
    // Build the element.
    pub fn build(origin_id: CompetitionId, highest_position: u8, lowest_position: u8, team_seeds: Seed, stats_carry_over: bool) -> Self {
        Self {
            origin_id,
            highest_position,
            lowest_position,
            team_seeds,
            stats_carry_over,

            ..Default::default()
        }
    }

    async fn save(&self, db: &Db, destination_id: CompetitionId) {
        sqlx::query(
            "INSERT INTO CompConnection
            (origin_id, destination_id, highest_position, lowest_position, team_seeds, stats_carry_over)
            VALUES ($1, $2, $3, $4, $5, $6)"
        ).bind(self.origin_id)
        .bind(destination_id)
        .bind(self.highest_position)
        .bind(self.lowest_position)
        .bind(self.team_seeds)
        .bind(self.stats_carry_over)
        .execute(db).await.unwrap();
    }

    async fn destination(&self, db: &Db) -> Competition {
        sqlx::query_as(
            "SELECT * FROM Competition WHERE id = $1"
        ).bind(self.destination_id)
        .fetch_one(db).await.unwrap()
    }

    // Send teams onwards to the next stage.
    async fn send_teams(&self, db: &Db, teams: &[TeamSeason]) {
        let mut teamdata = (self.highest_position - 1..self.lowest_position).map(|i| {
            let seed = match self.team_seeds {
                Seed::GetFromPosition => i + 1,
                Seed::Preserve => teams[i as usize].seed,
                _ => panic!(),
            };

            let team = if self.stats_carry_over {
                let mut t = teams[i as usize].clone();
                t.seed = seed;
                t
            } else {
                TeamSeason::build(teams[i as usize].team_id, seed)
            };
            team
        }).collect();

        self.destination(db).await.setup_season(db, &mut teamdata).await;
    }
}