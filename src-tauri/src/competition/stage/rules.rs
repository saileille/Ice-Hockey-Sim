#[derive(Default, PartialEq)]
pub struct RoundRobin {
    pub rounds: u8, // How many times each team plays one another.
    pub extra_matches: u8,  // How many matches should be scheduled in addition to rounds.
    points_for_win: u8,
    points_for_draw: u8,
    points_for_loss: u8,
    points_for_ot_win: u8,
    points_for_ot_loss: u8,
}

// Basics
impl RoundRobin {
    pub fn build(rounds: u8, extra_matches: u8) -> Self {
        let mut rr: Self = Self::default();
        rr.rounds = rounds;
        rr.extra_matches = extra_matches;
        return rr;
    }

    // Make sure the round robin rules do not have illegal values.
    pub fn is_valid(&self) -> bool {
        self.rounds != 0 || self.extra_matches != 0
    }
}

#[derive(Default, PartialEq)]
pub struct Knockout {
    wins_required: u8,
}

// Basics
impl Knockout {
    // Make sure knockout rules do not have illegal values.
    pub fn is_valid(&self) -> bool {
        return self.wins_required != 0
    }
}