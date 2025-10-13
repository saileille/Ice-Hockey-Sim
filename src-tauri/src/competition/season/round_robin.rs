// Round robin specific seasonal stuff.

use crate::{competition::{format, season::{team::TeamCompData, Season}, Competition}, team::Team, types::convert};

#[derive(Debug)]
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

    // Get the round robin standings.
    pub fn display_standings(&mut self) -> String {
        let comp = Competition::fetch_from_db(&self.comp_id).unwrap();
        self.rank_teams(&comp);
        let rr = comp.get_round_robin_format();

        let mut s = "Rank\tName\tG\tW\tOTW\tD\tOTL\tL\tGF\tGA\tDiff\tPts".to_string();
        for (i, team) in self.teams.iter().enumerate() {
            s += &format!("\n{}.\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                i + 1,
                &Team::fetch_from_db(&team.team_id).name,
                team.get_game_count(),
                team.regular_wins,
                team.ot_wins,
                team.draws,
                team.ot_losses,
                team.regular_losses,
                team.goals_scored,
                team.goals_conceded,
                team.get_goal_difference(),
                team.get_points(&rr)
            );
        }

        return s;
    }
}