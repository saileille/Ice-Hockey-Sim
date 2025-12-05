mod game;
pub mod season;
pub mod team;

use serde::Serialize;
use sqlx::{FromRow, Row, sqlite::SqliteRow};
use time::Date;

use crate::{logic::{competition::{self, Type, round_robin::RoundRobin as RoundRobinFormat}, types::{CompetitionId, Db}}, packages::screens::competition::season::SeasonPackage};

/*type Team = {
    id: number,
    name: string,
    rank: string,
    games: number,
    regular_wins: number,
    ot_wins: number,
    draws: number,
    ot_losses: number,
    regular_losses: number,
    goals_scored: number,
    goals_conceded: number,
    goal_difference: number,
    points: number,
};

type KnockoutRound = {
    pairs: Array<KnockoutPair>,
};

type KnockoutPair = {
    home: KnockoutTeam,
    away: KnockoutTeam,
};

type KnockoutTeam = {
    id: number,
    name: string,
    wins: number,
    seed: number,
};

type Game = {
    home: GameTeam,
    away: GameTeam,
    date: string,
    had_overtime: boolean,
};

type GameTeam = {
    id: number,
    name: string,
    seed: number,
    goals: number,
};

type Season = {
    teams: Array<Team>,
    knockout_rounds: Array<KnockoutRound>,
    upcoming_games: Array<Game>,
    played_games: Array<Game>
};

type Competition = {
    season: Season,
    comp_nav: Array<Array<[number, string]>>,
    competition_type: CompetitionType
}; */

// For more optimised queries, I suppose.
#[derive(Serialize)]
pub struct Package {
    season: SeasonPackage,
    comp_nav: Vec<Vec<(CompetitionId, String)>>,
    competition_type: competition::Type,
}

impl FromRow<'_, SqliteRow> for Package {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        Ok(Self {
            season: SeasonPackage::from_row(row)?,
            comp_nav: Vec::new(),
            competition_type: row.try_get("comp_type")?,
        })
    }
}

impl Package {
    pub async fn build(db: &Db, today: Date, comp_id: CompetitionId) -> Self {
        sqlx::query_as(
            "SELECT
            Competition.id AS competition_id, parent_id, comp_name, comp_type, Season.id,
            RoundRobinFormat.*
            FROM Competition

            INNER JOIN Season ON Season.comp_id = Competition.id
            LEFT JOIN RoundRobinFormat ON RoundRobinFormat.comp_id = Competition.id
            LEFT JOIN

            WHERE Competition.id = $1
            GROUP BY Season.comp_id
            ORDER BY Season.id DESC"
        ).bind(comp_id)
        .fetch_one(db).await.unwrap()
    }
}