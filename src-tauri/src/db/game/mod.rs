mod event;
mod team;

use futures::TryStreamExt as _;
use sqlx::{FromRow, Row, sqlite::SqliteRow};
use time::Date;

use crate::logic::{competition::Competition, game::{Game, Rules, event::Shot, team::TeamGame}, types::{CompetitionId, Db, GameId, SeasonId, TeamId}};

impl FromRow<'_, SqliteRow> for Game {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        let seconds = row.try_get("clock")?;
        let rules = Rules::from_row(row)?;

        Ok(Self {
            id: row.try_get("id")?,
            date: row.try_get("date")?,
            clock: rules.clock_from_seconds(seconds),
            season_id: row.try_get("season_id")?,
            rules,
            home: TeamGame::custom_from_row(row, "home")?,
            away: TeamGame::custom_from_row(row, "away")?,
        })
    }
}

impl Game {
    pub const SELECT_QUERY: &str = "
    SELECT GameRules.*, Game.*, home_lineup, away_lineup FROM Game

    LEFT JOIN (
        SELECT
        game_id, team_id, TeamGame.lineup AS home_lineup
        FROM TeamGame
    ) Home ON Home.game_id = Game.id AND Home.team_id = Game.home_id

    LEFT JOIN (
        SELECT
        game_id, team_id, TeamGame.lineup AS away_lineup
        FROM TeamGame
    ) Away ON Away.game_id = Game.id AND Away.team_id = Game.away_id

    INNER JOIN Season ON Season.id = Game.season_id
    INNER JOIN GameRules ON GameRules.comp_id = Season.comp_id";

    async fn build_from_db(db: &Db, id: GameId) -> Self {
        let query = Self::SELECT_QUERY;
        let mut game: Self = sqlx::query_as(format!("
            {query}
            WHERE Game.id = $1",
        ).as_str())
        .bind(id)
        .fetch_one(db).await.unwrap();

        game.fetch_shots(db).await;
        return game;
    }

    // Get all shots of this match.
    pub async fn fetch_shots(&mut self, db: &Db) {
        let mut rows = sqlx::query(
            "SELECT * FROM GameEvent
            INNER JOIN ShotEvent ON event_id = id
            WHERE game_id = $1"
        ).bind(self.id)
        .fetch(db);

        // Give each shot to the either team's struct.
        while let Some(row) = rows.try_next().await.unwrap() {
            let shot = Shot::from_row(&row).unwrap();
            if shot.event.target_team_id == self.home.team_id {
                self.home.shots.push(shot);
            }
            else {
                self.away.shots.push(shot);
            }
        }
    }

    pub async fn save(db: &Db, home_id: TeamId, away_id: TeamId, season_id: SeasonId, date: Date) {
        let id = sqlx::query_scalar(
            "INSERT INTO Game
            (date, home_id, away_id, season_id)
            VALUES ($1, $2, $3, $4)
            RETURNING id"
        ).bind(date)
        .bind(home_id)
        .bind(away_id)
        .bind(season_id)
        .fetch_one(db).await.unwrap();

        TeamGame::save(db, id, home_id).await;
        TeamGame::save(db, id, away_id).await;
    }

    // Save the match result in the database.
    pub async fn overwrite(&mut self, db: &Db) {
        sqlx::query(
            "UPDATE Game SET clock = $1
            WHERE id = $2"
        ).bind(self.total_seconds())
        .bind(self.id)
        .execute(db).await.unwrap();

        self.home.overwrite(db).await;
        self.away.overwrite(db).await;
    }

    pub async fn competition(&self, db: &Db) -> Competition {
        sqlx::query_as(
            "SELECT Competition.* FROM Competition
            INNER JOIN Season ON Season.comp_id = Competition.id
            WHERE Season.id = $1"
        ).bind(self.season_id)
        .fetch_one(db).await.unwrap()
    }
}

impl Rules {
    pub async fn save(&mut self, db: &Db) {
        self.id = sqlx::query_scalar(
            "INSERT INTO GameRules
            (periods, period_length, overtime_length, continuous_overtime)
            VALUES ($1, $2, $3, $4)
            RETURNING id"
        ).bind(self.periods)
        .bind(self.period_length)
        .bind(self.overtime_length)
        .bind(self.continuous_overtime)
        .fetch_one(db).await.unwrap();
    }
}