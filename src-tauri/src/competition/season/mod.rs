// Seasons represent a single iteration of a particular competition.
pub mod team;
pub mod round_robin;
pub mod knockout_round;
pub mod ranking;
mod schedule_generator;

use std::num::NonZero;

use serde_json::json;
use sqlx::FromRow;
use time::Date;

use crate::{competition::{Competition, season::{knockout_round::KnockoutRound as KnockoutRoundSeason, round_robin::RoundRobin as RoundRobinSeason, team::TeamSeason}}, match_event::Game, types::{CompetitionId, Db, SeasonId, TeamId}};

#[derive(Debug)]
#[derive(Clone)]
#[derive(FromRow)]
pub struct Season {
    pub id: SeasonId,
    pub comp_id: CompetitionId,
    #[sqlx(rename = "season_name")]
    name: String,   // Years during which the season takes place.
    start_date: Date,
    pub end_date: Date,
    #[sqlx(json(nullable))]
    pub round_robin: Option<RoundRobinSeason>,
    #[sqlx(json(nullable))]
    pub knockout_round: Option<KnockoutRoundSeason>,

    // Helper for easily checking if the season is over.
    pub is_over: bool,
}

impl Default for Season {
    fn default() -> Self {
        Self {
            id: SeasonId::default(),
            comp_id: CompetitionId::default(),
            name: String::default(),
            start_date: Date::MIN,
            end_date: Date::MIN,
            round_robin: None,
            knockout_round: None,
            is_over: bool::default(),
        }
    }
}

impl Season {
    // Get the next ID to use.
    async fn next_id(db: &Db) -> SeasonId {
        let max: Option<SeasonId> = sqlx::query_scalar("SELECT max(id) FROM Season").fetch_one(db).await.unwrap();
        match max {
            Some(n) => n + 1,
            _ => 1,
        }
    }

    // Build an element.
    async fn build(db: &Db, comp: &Competition) -> Self {
        let mut season = Self {
            id: Self::next_id(db).await,
            comp_id: comp.id,
            start_date: comp.season_window.get_next_start_date(db).await,
            end_date: comp.season_window.get_next_end_date(db).await,

            ..Default::default()
        };

        season.name = if season.start_date.year() == season.end_date.year() {
            season.start_date.year().to_string()
        }
        else {
            format!("{}-{}", season.start_date.year(), season.end_date.year())
        };


        let format = &comp.format.as_ref();
        if format.is_none() { return season; }

        if format.unwrap().round_robin.as_ref().is_some() {
            season.round_robin = Some(RoundRobinSeason::build());
        }
        else if format.unwrap().knockout_round.as_ref().is_some() {
            season.knockout_round = Some(KnockoutRoundSeason::build());
        }
        else {
            panic!("{}\nformat: {:#?}", comp.name, comp.format);
        }

        return season;
    }

    // Build a season and save it to the database.
    // Also build seasons for all possible child competitions.
    pub async fn build_and_save(db: &Db, comp: &Competition, teams: &[TeamId]) -> Self {
        let mut season = Self::build(db, comp).await;

        season.save_new(db, teams).await;
        return season;
    }

    // Save a season to the database for the first time.
    async fn save_new(&mut self, db: &Db, teams: &[TeamId]) {
        self.save(db).await;

        for id in teams {
            sqlx::query(
                "INSERT INTO TeamSeason
                (team_id, season_id, seed, rank, regular_wins, ot_wins, draws, ot_losses, regular_losses, goals_scored, goals_conceded)
                VALUES ($1, $2, 0, 1, 0, 0, 0, 0, 0, 0, 0)"
            ).bind(NonZero::new(*id).unwrap())
            .bind(NonZero::new(self.id).unwrap())
            .execute(db).await.unwrap();
        }
    }

    // Save the Season to database.
    pub async fn save(&self, db: &Db) {
        sqlx::query(
            "INSERT INTO Season
            (id, comp_id, season_name, start_date, end_date, round_robin, knockout_round, is_over)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
        ).bind(NonZero::new(self.id).unwrap())
        .bind(NonZero::new(self.comp_id).unwrap())
        .bind(self.name.as_str())
        .bind(self.start_date)
        .bind(self.end_date)
        .bind(&self.round_robin)
        .bind(&self.knockout_round)
        .bind(self.is_over)
        .execute(db).await.unwrap();
    }

    // Get the competition of the season.
    async fn competition(&self, db: &Db) -> Competition {
        Competition::fetch_from_db(db, self.comp_id).await
    }

    // Get the full name of the season, with all parent competition names included.
    async fn _full_name(&self, db: &Db) -> String {
        let comp_name = self.competition(db).await.full_name(db).await;
        format!("{} {}", comp_name, self.name)
    }

    // Get some nice JSON for a competition screen.
    pub async fn comp_screen_package(&self, db: &Db, comp: &Competition) -> serde_json::Value {
        let mut teams = Vec::new();
        for team in self.teams(db).await {
            teams.push(team.comp_screen_package(db, comp).await);
        }

        let future_games = self.today_and_future_games(db).await;
        let mut future_games_json = Vec::new();
        for mut game in future_games {
            future_games_json.push(game.comp_screen_package(db).await);
        }

        let played_games = self.past_games(db).await;
        let mut played_games_json = Vec::new();
        for mut game in played_games {
            played_games_json.push(game.comp_screen_package(db).await);
        }

        let knockout_round = if self.knockout_round.is_none() {
            serde_json::Value::Null
        }
        else {
            self.knockout_round.as_ref().unwrap().comp_screen_package(db).await
        };

        json!({
            "name": self.name,
            "teams": teams,
            "knockout_round": knockout_round,
            "upcoming_games": future_games_json,
            "played_games": played_games_json,
        })
    }

    // Get all teams participating in the season.
    pub async fn teams(&self, db: &Db) -> Vec<TeamSeason> {
        sqlx::query_as(
            "SELECT * FROM TeamSeason
            WHERE season_id = $1
            ORDER BY rank ASC"
        ).bind(self.id)
        .fetch_all(db).await.unwrap()
    }

    pub async fn team_ids(&self, db: &Db) -> Vec<TeamId> {
        sqlx::query_scalar(
            "SELECT team_id FROM TeamSeason
            WHERE season_id = $1
            ORDER BY rank ASC"
        ).bind(self.id)
        .fetch_all(db).await.unwrap()
    }

    // Get the season data of a team with the given ID.
    pub async fn team_with_id(&self, db: &Db, id: TeamId) -> TeamSeason {
        sqlx::query_as(
            "SELECT * FROM TeamSeason
            WHERE team_id = $1 AND season_id = $2"
        ).bind(id)
        .bind(self.id)
        .fetch_one(db).await.unwrap()
    }

    // Get the amount of teams in the season.
    pub async fn no_of_teams(&self, db: &Db) -> u8 {
        sqlx::query_scalar(
            "SELECT COUNT(*) FROM TeamSeason WHERE season_id = $1"
        ).bind(self.id)
        .fetch_one(db).await.unwrap()
    }

    // Get all games for this competition that are played today.
    async fn games_today(&self, db: &Db) -> Vec<Game> {
        sqlx::query_as(
            "SELECT * FROM Game
            WHERE date = (
                SELECT value_data FROM KeyValue WHERE key_name = 'today'
            ) AND season_id = $1"
        ).bind(self.id)
        .fetch_all(db).await.unwrap()
    }

    // Get all games that have been played on past dates.
    pub async fn past_games(&self, db: &Db) -> Vec<Game> {
        sqlx::query_as(
            "SELECT * FROM Game
            WHERE unixepoch(date) < (
                SELECT unixepoch(value_data) FROM KeyValue WHERE key_name = 'today'
            ) AND season_id = $1
            ORDER BY unixepoch(date) DESC"
        ).bind(self.id)
        .fetch_all(db).await.unwrap()
    }

    // Get all games that are played on future dates.
    async fn future_games(&self, db: &Db) -> Vec<Game> {
        sqlx::query_as(
            "SELECT * FROM Game
            WHERE unixepoch(date) > (
                SELECT unixepoch(value_data) FROM KeyValue WHERE key_name = 'today'
            ) AND season_id = $1"
        ).bind(self.id)
        .fetch_all(db).await.unwrap()
    }

    pub async fn today_and_future_games(&self, db: &Db) -> Vec<Game> {
        sqlx::query_as(
            "SELECT * FROM Game
            WHERE unixepoch(date) >= (
                SELECT unixepoch(value_data) FROM KeyValue WHERE key_name = 'today'
            ) AND season_id = $1
            ORDER BY unixepoch(date) ASC"
        ).bind(self.id)
        .fetch_all(db).await.unwrap()
    }

    // Finalise the creation of a season for a particular competition.
    pub async fn setup(&mut self, db: &Db, comp: &Competition) {
        if self.round_robin.is_some() {
            self.setup_round_robin(db, comp).await;
        }
        else if self.knockout_round.is_some() {
            self.setup_knockout(db, comp, self.id).await;
        }

        // In this case the competition must have child competitions, so set them up instead.
        else {
            let children = comp.children(db).await;
            let mut teams = Vec::new();
            for (i, comp) in children.into_iter().enumerate() {
                if i == 0 {
                    // Set up all the teams here if the child competition is the first one.
                    // Teams that cannot be added will go to the next rounds.
                    // Does not support group competitions yet.
                    teams = self.teams(db).await;
                }
                comp.setup_season(db, &mut teams).await;
            }
        }
    }

    // Set up a round robin season.
    async fn setup_round_robin(&mut self, db: &Db, comp: &Competition) {
        self.generate_schedule(db, comp).await;
    }

    // Set up a knockout season.
    async fn setup_knockout(&mut self, db: &Db, comp: &Competition, season_id: SeasonId) {
        let teams = comp.current_season_teamdata(db).await;
        self.knockout_round.as_mut().unwrap().setup(db, &teams, self.start_date, self.end_date, comp, season_id).await;
    }

    // Simulate the games for this day.
    pub async fn simulate_day(&self, db: &Db) {
        let games = self.games_today(db).await;
        if games.is_empty() { return }

        for mut game in games {
            game.play(db).await;
        }

        let mut o_comp = Some(self.competition(db).await);
        for i in 0.. {
            let comp = o_comp.as_ref().unwrap();
            let mut season = match i {
                0 => self.clone(),
                _ => comp.current_season(db).await,
            };

            season.check_if_over(db, &comp).await;
            season.rank_teams(db, &comp).await;
            season.save(db).await;

            o_comp = comp.parent(db).await;
            if o_comp.is_none() { return }
        }
    }

    // Check if the season has ended, and react appropriately.
    // Return whether over or not.
    pub async fn check_if_over(&mut self, db: &Db, comp: &Competition) -> bool {
        // No need to do more.
        if self.is_over { return true; }

        let upcoming_games = self.future_games(db).await;
        if self.round_robin.is_some() {
            if !upcoming_games.is_empty() { return false; }
        }

        else if self.knockout_round.is_some() {
            if !self.knockout_round.as_mut().unwrap().check_if_over(db, comp).await {
                return false;
            }
        }

        // Check for a parent competition.
        // TODO: Make an async recursion happen somehow...?
        else {
            for child_comp in comp.children(db).await {
                let mut season = child_comp.current_season(db).await;
                if !Box::pin(season.check_if_over(db, &child_comp)).await {
                    return false;
                }
            }
        }

        self.is_over = true;
        self.do_post_season_tasks(db, comp).await;

        return true;
    }

    // Do post-season tasks for any kind of competition.
    async fn do_post_season_tasks(&mut self, db: &Db, comp: &Competition) {
        self.rank_teams(db, comp).await;
        for connection in comp.connections_to(db).await {
            connection.send_teams(db, &self.teams(db).await).await;
        }
    }
}