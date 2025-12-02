// Functions exclusive to knockout stages.
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::logic::types::CompetitionId;

#[derive(Debug, Serialize, Deserialize)]
#[derive(Clone, Default)]
#[derive(FromRow)]
pub struct KnockoutRound {
    comp_id: CompetitionId,
    pub wins_required: u8,
    pub maximum_games: u8,
}

impl KnockoutRound {
    // Build the element.
    pub fn build(wins_required: u8) -> Self {
        KnockoutRound {
            wins_required,

            ..Default::default()
        }
    }
}