use serde::Serialize;
use sqlx::{FromRow, Row, sqlite::SqliteRow};
use time::Date;

use crate::logic::{game::event::Shot, time::iso_date_format, types::TeamId};

#[derive(FromRow)]
#[derive(Serialize)]
pub struct GamePackage {
    pub home: TeamPackage,
    pub away: TeamPackage,
    #[serde(with = "iso_date_format")]
    pub date: Date,
    had_overtime: bool,
    is_over: bool,
}

impl FromRow<'_, SqliteRow> for GamePackage {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        Ok(Self {
            home: TeamPackage::custom_from_row(row, "home")?,
            away: TeamPackage::custom_from_row(row, "away")?,
            date: row.try_get("date")?,
            had_overtime: row.try_get("had_overtime")?,
            is_over: row.try_get("is_over")?,
        })
    }
}

impl GamePackage {
    pub fn select_query() -> String {
        let home_and_away = TeamPackage::home_away_queries();
        format!(
            "SELECT (GameRules.total_length < Game.clock) AS had_overtime, (Game.clock > 0) AS is_over, Game.date,
            home_id, home_seed, home_name, home_goals,
            away_id, away_seed, away_name, away_goals
            FROM Game

            {home_and_away}

            INNER JOIN Season ON Season.id = Game.season_id
            INNER JOIN GameRules ON GameRules.comp_id = Season.comp_id"
        )
}
}

#[derive(Serialize)]
pub struct TeamPackage {
    id: TeamId,
    name: String,
    seed: u8,
    goals: u8,
}

impl TeamPackage {
    pub fn custom_from_row(row: &SqliteRow, home_away: &str) -> sqlx::Result<Self> {
        Ok(Self {
            id: row.try_get(format!("{home_away}_id").as_str())?,
            name: row.try_get(format!("{home_away}_name").as_str())?,
            seed: row.try_get(format!("{home_away}_seed").as_str())?,
            goals: row.try_get(format!("{home_away}_goals").as_str())?,
        })
    }

    fn home_away_queries() -> String {
        format!("{} {}", Self::home_away_query("home"), Self::home_away_query("away"))
    }

    fn home_away_query(home_away: &str) -> String {
        let goals = Shot::QUERY_TEAM_GOALS_IN_MATCH;
        format!("
            LEFT JOIN (
                SELECT
                game_id, TeamGame.team_id, seed AS {home_away}_seed,
                full_name AS {home_away}_name,
                {goals} AS {home_away}_goals,

                TeamSeason.season_id
                FROM TeamGame
                INNER JOIN Team ON Team.id = TeamGame.team_id

                INNER JOIN TeamSeason ON TeamSeason.team_id = TeamGame.team_id

            ) {home_away} ON {home_away}.game_id = Game.id AND {home_away}.team_id = Game.{home_away}_id
            AND {home_away}.season_id = Game.season_id
        ")
    }
}