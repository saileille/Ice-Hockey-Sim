// Round robin specific seasonal stuff.

use crate::{competition::season::Season, types::convert};

#[derive(Debug, serde::Serialize)]
#[derive(Default, Clone)]
pub struct RoundRobin {

}

impl RoundRobin {
    pub fn build() -> Self {
        Self::default()
    }
}

impl Season {
    // Get the amount of actual games each team plays in round robin.
    pub fn get_matches_per_team(&self) -> u8 {
        let matches = convert::usize_to_f64(self.get_all_games().len());
        let teams = convert::usize_to_f64(self.teams.len());

        return convert::f64_to_u8(matches / teams * 2.0);
    }
}