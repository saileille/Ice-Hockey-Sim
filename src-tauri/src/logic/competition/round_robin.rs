// The round robin struct and methods for round-robin stages.
use serde::Serialize;
use sqlx::FromRow;

use crate::logic::{competition::season::Season, types::{CompetitionId, Db}};

#[derive(Default, Clone, PartialEq, Debug)]
pub enum MatchGenType {
    #[default] Null,
    _MatchCount,
    _Random,
    Alternating,
}

#[derive(Debug, Default, Clone)]
#[derive(FromRow)]
#[derive(Serialize)]
pub struct RoundRobin {
    comp_id: CompetitionId,
    pub rounds: u8, // How many times each team plays one another.
    pub extra_matches: u8,  // How many matches should be scheduled in addition to rounds.
    pub points_for_win: u8,
    pub points_for_ot_win: u8,
    pub points_for_draw: u8,
    pub points_for_ot_loss: u8,
    pub points_for_loss: u8,
}

impl RoundRobin {
    pub const MATCH_GEN_TYPE: MatchGenType = MatchGenType::Alternating;

    pub fn build(rounds: u8, extra_matches: u8, points_for_win: u8, points_for_ot_win: u8, points_for_draw: u8, points_for_ot_loss: u8, points_for_loss: u8) -> Self {
        Self {
            rounds: rounds,
            extra_matches: extra_matches,
            points_for_win: points_for_win,
            points_for_ot_win: points_for_ot_win,
            points_for_draw: points_for_draw,
            points_for_ot_loss: points_for_ot_loss,
            points_for_loss: points_for_loss,

            ..Default::default()
        }
    }

    // Get how many matches each team should play.
    pub async fn theoretical_matches_per_team(&self, db: &Db, season: &Season) -> u8 {
        self.round_length(db, season).await * self.rounds
        + self.extra_matches
    }

    // Get how many matches each team has to play to face each team once.
    pub async fn round_length(&self, db: &Db, season: &Season) -> u8 {
        season.no_of_teams(db).await - 1
    }
}