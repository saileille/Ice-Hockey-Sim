// Functions exclusive to knockout stages.
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::logic::types::{Db, KnockoutRoundFormatId};

#[derive(Debug, Serialize, Deserialize)]
#[derive(Clone, Default)]
#[derive(FromRow)]
pub struct KnockoutRound {
    pub id: KnockoutRoundFormatId,
    pub wins_required: u8,
    pub maximum_games: u8,
}

impl KnockoutRound {
    // Build the element.
    fn build(wins_required: u8) -> Self {
        KnockoutRound {
            wins_required,

            ..Default::default()
        }
    }

    pub async fn build_and_save(db: &Db, wins_required: u8) -> Self {
        let mut kr = Self::build(wins_required);
        kr.save(db).await;
        return kr;
    }
}