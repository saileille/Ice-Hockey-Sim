// The round robin struct and methods for round-robin stages.
mod schedule_validator;

use serde::{Deserialize, Serialize};

use crate::{competition::season::Season, types::Db};

#[derive(Default, Clone, PartialEq, Debug)]
pub enum MatchGenType {
    #[default] Null,
    _MatchCount,
    _Random,
    Alternating,
}

#[derive(Debug, Serialize, Deserialize)]
#[derive(Default, Clone)]
pub struct RoundRobin {
    pub rounds: u8, // How many times each team plays one another.
    pub extra_matches: u8,  // How many matches should be scheduled in addition to rounds.
    pub points_for_win: u8,
    pub points_for_ot_win: u8,
    pub points_for_draw: u8,
    pub points_for_ot_loss: u8,
    pub points_for_loss: u8,
}

// Basics
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
        }
    }
}

impl RoundRobin {
    // Get how many matches each team should play.
    pub async fn get_theoretical_matches_per_team(&self, db: &Db, season: &Season) -> u8 {
        self.get_round_length(db, season).await * self.rounds
        + self.extra_matches
    }

    // Get how many matches each team has to play to face each team once.
    pub async fn get_round_length(&self, db: &Db, season: &Season) -> u8 {
        season.no_of_teams(db).await - 1
    }
}