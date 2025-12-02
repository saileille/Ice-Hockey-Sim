mod knockout_round;
mod round_robin;
mod team;

use crate::logic::{competition::season::{Season, knockout_round::KnockoutRound, team::TeamSeason}, game::Game, types::{CompetitionId, Db, SeasonId, TeamId}};

impl Season {
    // Save a season to the database for the first time.
    pub async fn save_new(&mut self, db: &Db, teams: &[TeamId]) {
        self.save(db).await;

        for id in teams {
            sqlx::query(
                "INSERT INTO TeamSeason
                (team_id, season_id, seed, ranking, regular_wins, ot_wins, draws, ot_losses, regular_losses, goals_scored, goals_conceded)
                VALUES ($1, $2, 0, 1, 0, 0, 0, 0, 0, 0, 0)"
            ).bind(id)
            .bind(self.id)
            .execute(db).await.unwrap();
        }
    }

    // Save the Season to database.
    pub async fn save(&mut self, db: &Db) {
        self.id = sqlx::query_scalar(
            "INSERT INTO Season
            (comp_id, season_name, start_date, end_date, round_robin, knockout_round, is_over)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id"
        ).bind(self.comp_id)
        .bind(self.name.as_str())
        .bind(self.start_date)
        .bind(self.end_date)
        .bind(&self.round_robin)
        .bind(&self.knockout_round)
        .bind(self.is_over)
        .fetch_one(db).await.unwrap();
    }

    // Get all teams participating in the season.
    pub async fn teams(&self, db: &Db) -> Vec<TeamSeason> {
        sqlx::query_as(
            "SELECT * FROM TeamSeason
            WHERE season_id = $1
            ORDER BY ranking ASC"
        ).bind(self.id)
        .fetch_all(db).await.unwrap()
    }

    pub async fn team_ids(&self, db: &Db) -> Vec<TeamId> {
        sqlx::query_scalar(
            "SELECT team_id FROM TeamSeason
            WHERE season_id = $1
            ORDER BY ranking ASC"
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
    pub async fn games_today(&self, db: &Db) -> Vec<Game> {
        let mut games: Vec<Game> = sqlx::query_as(format!(
            "{}
            WHERE date = (
                SELECT value_data FROM KeyValue WHERE key_name = 'today'
            ) AND season_id = $1",
        Game::SELECT_QUERY).as_str())
        .bind(self.id)
        .fetch_all(db).await.unwrap();

        for game in games.iter_mut() {
            game.fetch_shots(db).await;
        }

        return games;
    }

    // Get all games that have been played on past dates.
    pub async fn past_games(&self, db: &Db) -> Vec<Game> {
        let mut games: Vec<Game> = sqlx::query_as(format!(
            "{}
            WHERE unixepoch(date) < (
                SELECT unixepoch(value_data) FROM KeyValue WHERE key_name = 'today'
            ) AND season_id = $1
            ORDER BY unixepoch(date) DESC",
        Game::SELECT_QUERY).as_str())
        .bind(self.id)
        .fetch_all(db).await.unwrap();

        for game in games.iter_mut() {
            game.fetch_shots(db).await;
        }

        return games;
    }

    // Get all games that are played on future dates.
    pub async fn future_games(&self, db: &Db) -> Vec<Game> {
        let mut games: Vec<Game> = sqlx::query_as(format!(
            "{}
            WHERE unixepoch(date) > (
                SELECT unixepoch(value_data) FROM KeyValue WHERE key_name = 'today'
            ) AND season_id = $1",
        Game::SELECT_QUERY).as_str())
        .bind(self.id)
        .fetch_all(db).await.unwrap();

        for game in games.iter_mut() {
            game.fetch_shots(db).await;
        }

        return games;
    }

    pub async fn today_and_future_games(&self, db: &Db) -> Vec<Game> {
        let mut games: Vec<Game> = sqlx::query_as(format!(
            "{}
            WHERE unixepoch(date) >= (
                SELECT unixepoch(value_data) FROM KeyValue WHERE key_name = 'today'
            ) AND season_id = $1
            ORDER BY unixepoch(date) ASC",
        Game::SELECT_QUERY).as_str())
        .bind(self.id)
        .fetch_all(db).await.unwrap();

        for game in games.iter_mut() {
            game.fetch_shots(db).await;
        }

        return games;
    }

    pub async fn knockout_round_from_id(db: &Db, id: SeasonId) -> KnockoutRound {
        sqlx::query_scalar(
            "SELECT knockout_round FROM Season
            WHERE id = $1"
        ).bind(id)
        .fetch_one(db).await.unwrap()
    }

    // Get the child knockout rounds from competition ID.
    // Only current season for now.
    pub async fn child_knockout_rounds_from_id(db: &Db, id: CompetitionId) -> Vec<KnockoutRound> {
        sqlx::query_scalar(
            "SELECT knockout_round FROM Season
            INNER JOIN Competition ON Competition.id = Season.comp_id
            WHERE Competition.parent_id = $1
            GROUP BY Season.comp_id
            ORDER BY Competition.id ASC, Season.id DESC"
        ).bind(id)
        .fetch_all(db).await.unwrap()
    }

    // Save the team rankings.
    pub async fn save_rank(&self, db: &Db, teams: Vec<TeamSeason>) {
        for (i, team) in teams.into_iter().enumerate() {
            sqlx::query(
                "UPDATE TeamSeason SET ranking = $1
                WHERE team_id = $2 AND season_id = $3"
            ).bind(i as u8 + 1)
            .bind(team.team_id)
            .bind(self.id)
            .execute(db).await.unwrap();
        }
    }

}