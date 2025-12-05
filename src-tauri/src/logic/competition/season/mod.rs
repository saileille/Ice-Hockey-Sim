// Seasons represent a single iteration of a particular competition.
pub mod team;
pub mod round_robin;
pub mod knockout_round;
pub mod ranking;
mod schedule_generator;

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::Date;

use crate::logic::{competition::{self, Competition, season::{knockout_round::KnockoutRound, round_robin::RoundRobin}}, types::{CompetitionId, Db, SeasonId, TeamId}};

#[derive(Debug)]
#[derive(Clone)]
#[derive(Serialize, Deserialize)]
#[derive(FromRow)]
pub struct Season {
    pub id: SeasonId,
    pub comp_id: CompetitionId,
    #[sqlx(rename = "season_name")]
    pub name: String,   // Years during which the season takes place.
    pub start_date: Date,
    pub end_date: Date,
    #[sqlx(json(nullable))]
    pub round_robin: Option<RoundRobin>,

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
            is_over: bool::default(),
        }
    }
}

impl Season {
    // Build an element.
    fn build(today: Date, comp: &Competition) -> Self {
        let mut season = Self {
            comp_id: comp.id,
            start_date: comp.season_window.next_start_date(today),
            end_date: comp.season_window.next_end_date(today),

            ..Default::default()
        };

        season.name = if season.start_date.year() == season.end_date.year() {
            season.start_date.year().to_string()
        }
        else {
            format!("{}-{}", season.start_date.year(), season.end_date.year())
        };


        if comp.comp_type == competition::Type::Null
        || comp.comp_type == competition::Type::Tournament {
            return season;
        }

        if comp.comp_type == competition::Type::RoundRobin {
            season.round_robin = Some(RoundRobin::build());
        }
        else if comp.comp_type == competition::Type::KnockoutRound {

        }
        else {
            panic!("{}\nformat: {:?}", comp.name, comp.comp_type);
        }

        return season;
    }

    // Build a season and save it to the database.
    // Also build seasons for all possible child competitions.
    pub async fn build_and_save(db: &Db, today: Date, comp: &Competition, teams: &[TeamId]) -> Self {
        let mut season = Self::build(today, comp);

        season.save_new(db, teams).await;
        return season;
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

    // Finalise the creation of a season for a particular competition.
    pub async fn setup(&mut self, db: &Db, comp: &Competition) {
        if self.round_robin.is_some() {
            self.setup_round_robin(db, comp).await;
        }
        else if self.knockout_round.is_some() {
            self.setup_knockout(db, comp).await;
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
    async fn setup_knockout(&mut self, db: &Db, comp: &Competition) {
        let teams = comp.current_season_teamdata(db).await;
        self.knockout_round.as_mut().unwrap().setup(db, &teams, self.start_date, self.end_date, comp, self.id).await;
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