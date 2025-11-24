// Functions exclusive to knockout stages.

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[derive(Clone, Default)]
pub struct KnockoutRound {
    pub wins_required: u8,
}

impl KnockoutRound {
    // Build the element.
    pub fn build(wins_required: u8) -> Self {
        KnockoutRound { wins_required: wins_required }
    }

    // Get the amount of matches there can be in the round at most.
    pub fn get_maximum_matches_in_pair(&self) -> u8 {
        return self.wins_required * 2 - 1
    }
}