use futures::TryStreamExt as _;
use serde::Serialize;
use sqlx::{FromRow, Row, sqlite::SqliteRow};
use time::Date;

use crate::{logic::{competition::{self, round_robin::RoundRobin as RoundRobinFormat, season::{Season, knockout_round::KnockoutRound}}, types::{CompetitionId, Db, SeasonId}}, packages::screens::competition::{game::GamePackage, team::{KnockoutTeamPackage, TeamPackage}}};

#[derive(Serialize)]
pub struct SeasonPackage {
    teams: Vec<TeamPackage>,
    knockout_rounds: Vec<KnockoutRoundPackage>,
    upcoming_games: Vec<GamePackage>,
    played_games: Vec<GamePackage>,
}

impl FromRow<'_, SqliteRow> for SeasonPackage {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        Ok(Self {
            teams: Vec::new(),
            knockout_rounds: Vec::new(),
            upcoming_games: Vec::new(),
            played_games: Vec::new(),
        })
    }
}

#[derive(Serialize)]
pub struct KnockoutRoundPackage {
    name: String,
    pub pairs: Vec<KnockoutPairPackage>,
}

impl KnockoutRoundPackage {
    pub fn build(knockout_round: &KnockoutRound, comp_name: String) -> Self {
        Self {
            name: comp_name,
            // pairs: knockout_round.pairs.iter().map(|pair| pair.comp_screen_package()).collect(),
            pairs: Vec::new(),
        }
    }
}

#[derive(Serialize)]
pub struct KnockoutPairPackage {
    pub home: KnockoutTeamPackage,
    pub away: KnockoutTeamPackage,
}

impl FromRow<'_, SqliteRow> for KnockoutPairPackage {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        Ok(Self {
            home: KnockoutTeamPackage::custom_from_row(row, "home")?,
            away: KnockoutTeamPackage::custom_from_row(row, "away")?,
        })
    }
}

impl KnockoutRound {
    fn comp_package(&self) {

    }
}