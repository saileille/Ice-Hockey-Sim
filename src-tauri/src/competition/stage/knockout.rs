// Functions exclusive to knockout stages.

use std::ops::Range;
use crate::types::convert;
use super::{Stage, TeamData};

#[derive(Clone)]
pub struct Knockout {
    wins_required: u8,
    tree: Vec<KnockoutRound>,   // Consists of knockout rounds and pairs.
    teams_at_end: u8,   // Knockout rounds will continue until this many teams remain.
}

// Basics
impl Knockout {
    // Make sure knockout rules do not have illegal values.
    pub fn is_valid(&self) -> bool {
        return self.wins_required != 0
    }

    // Create a playoff tree.
    fn create_tree(&mut self, stage: &Stage) {
        let mut team_count: u8 = convert::usize_to_u8(stage.teams.len());
        while team_count > self.teams_at_end {
            let mut round: KnockoutRound = KnockoutRound::default();
            for _ in (Range {start: 0, end: team_count / 2}) {
                round.pairs.push(KnockoutPair::default());
            }
            self.tree.push(round);
            team_count /= 2;
        }
    }
}

#[derive(Clone, Default)]
struct KnockoutRound {
    pairs: Vec<KnockoutPair>,
}

#[derive(Default, Clone)]
struct KnockoutPair {
    home: TeamData,
    away: TeamData,
}