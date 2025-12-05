use ordinal::ToOrdinal as _;
use serde::Serialize;
use sqlx::{FromRow, Row, sqlite::SqliteRow};

use crate::logic::{competition::round_robin::RoundRobin as RoundRobinFormat, types::TeamId};

#[derive(Serialize)]
pub struct TeamPackage {
    id: TeamId,
    name: String,
    seed: u8,
    rank: String,
    games: u8,
    regular_wins: u8,
    ot_wins: u8,
    draws: u8,
    ot_losses: u8,
    regular_losses: u8,
    all_wins: u8,
    all_losses: u8,
    goals_scored: u16,
    goals_conceded: u16,
    goal_difference: i16,
    points: u8,
}

impl FromRow<'_, SqliteRow> for TeamPackage {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        let rank_u8: u8 = row.try_get("ranking")?;
        Ok(Self {
            id: row.try_get("team_id")?,
            name: row.try_get("full_name")?,
            seed: row.try_get("seed")?,
            rank: rank_u8.to_ordinal_string(),
            games: row.try_get("games")?,
            regular_wins: row.try_get("regular_wins")?,
            ot_wins: row.try_get("ot_wins")?,
            draws: row.try_get("draws")?,
            ot_losses: row.try_get("ot_losses")?,
            regular_losses: row.try_get("regular_losses")?,
            all_wins: row.try_get("all_wins")?,
            all_losses: row.try_get("all_losses")?,
            goals_scored: row.try_get("goals_scored")?,
            goals_conceded: row.try_get("goals_conceded")?,
            goal_difference: row.try_get("goal_difference")?,
            points: u8::default(),
        })
    }
}

impl TeamPackage {
    pub fn count_points(&mut self, rr_format: &RoundRobinFormat) {
        self.points =
            self.regular_wins * rr_format.points_for_win +
            self.ot_wins * rr_format.points_for_ot_win +
            self.draws * rr_format.points_for_draw +
            self.ot_losses * rr_format.points_for_ot_loss +
            self.regular_losses * rr_format.points_for_loss
        ;
    }
}

#[derive(Serialize)]
pub struct KnockoutTeamPackage {
    id: TeamId,
    name: String,
    wins: u8,
    seed: u8,
}

impl KnockoutTeamPackage {
    pub fn custom_from_row(row: &SqliteRow, home_away: &str) -> sqlx::Result<Self> {
        Ok(Self {
            id: row.try_get(format!("{home_away}_id").as_str())?,
            name: row.try_get(format!("{home_away}_full_name").as_str())?,
            seed: row.try_get(format!("{home_away}_seed").as_str())?,
            wins: row.try_get(format!("{home_away}_wins").as_str())?,
        })
    }

    // Temporary thing, do not use.
    /*pub fn build(team: &TeamSeason) -> Self {
        Self {
            id: team.team_id,
            name: String::new(),
            wins: team.all_wins,
            seed: team.seed,
        }
    }*/
}