// Functions exclusive to round robin stages.
use crate::{types::convert, team::Team, match_event::Game};
use super::{rules, Stage, TeamData};

impl Stage {
    // Get how many matches each team should play.
    pub fn get_theoretical_matches_per_team(&self) -> u8 {
        let rr: &rules::RoundRobin = self.round_robin.as_ref().unwrap();
        self.get_round_length() * rr.rounds
        + rr.extra_matches
    }

    // Check if the stage has a valid amount of matches.
    // Increase the matches by one if that is not the case.
    fn validate_match_amount(&mut self) {
        let mut rr: rules::RoundRobin = self.round_robin.as_ref().unwrap().clone();
        let matches_per_team: u8 = self.get_theoretical_matches_per_team();

        // Make sure there is at least one match on the stage per team.
        if matches_per_team == 0 {
            rr.extra_matches += 1
        }

        // Either the amount of teams or the matches per team must be even.
        if (self.teams.len() % 2 != 0) && (matches_per_team % 2 != 0) {
            rr.extra_matches += 1;
        }

        self.round_robin = Some(rr);
    }

    // Check if the match schedule went according to plan.
    pub fn had_successful_match_generation(&self) -> bool {
        self.get_theoretical_total_matches() == convert::usize_to_u16(Game::fetch_stage_matches(self.id).len())
    }

    // Get how many matches each team has to play to face each team once.
    pub fn get_round_length(&self) -> u8 {
        convert::usize_to_u8(self.teams.len() - 1)
    }

    // Get the teams in the order of the standings.
    fn get_sorted_teams(&self) -> Vec<TeamData> {
        let mut teams: Vec<TeamData> = self.teams.values().cloned().collect();
        let rules: &rules::RoundRobin = self.round_robin.as_ref().unwrap();

        teams.sort_by(|a, b| {
            b.get_points(rules).cmp(&a.get_points(rules))
            .then(b.get_goal_difference().cmp(&a.get_goal_difference()))
            .then(b.goals_scored.cmp(&a.goals_scored))
            .then(b.regular_wins.cmp(&a.regular_wins))
            .then(b.ot_wins.cmp(&a.ot_wins))
            .then(b.draws.cmp(&a.draws))
            .then(b.ot_losses.cmp(&a.ot_losses))
            .then(Team::fetch_from_db(&a.team_id).name.cmp(&Team::fetch_from_db(&b.team_id).name))
        });

        return teams;
    }

    // Get the amount of actual games each team plays.
    pub fn get_matches_per_team(&self) -> u8 {
        let matches: f64 = convert::usize_to_f64(Game::fetch_stage_matches(self.id).len());
        let teams: f64 = convert::usize_to_f64(self.teams.len());

        return convert::f64_to_u8(matches / teams * 2.0);
    }
}

// Testing.
impl Stage {
    // Get how many matches there should be in the stage in total.
    pub fn get_theoretical_total_matches(&self) -> u16 {
        return (self.get_theoretical_matches_per_team() as u16) * convert::usize_to_u16(self.teams.len()) / 2;
    }

    // Check if the stage has a valid amount of matches.
    // For testing purposes only. For in-game use, see validate_match_amount.
    pub fn has_valid_match_amount(&self) -> bool {
        (self.teams.len() % 2 == 0) || (self.get_theoretical_matches_per_team() % 2 == 0)
    }

    // Get the standings.
    pub fn display_standings(&self) -> String {
        let teams: Vec<TeamData> = self.get_sorted_teams();
        let rules: &rules::RoundRobin = self.round_robin.as_ref().unwrap();

        let mut s: String = "Rank\tName\tG\tW\tOTW\tD\tOTL\tL\tGF\tGA\tDiff\tPts".to_string();
        for (i, team) in teams.iter().enumerate() {
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
                team.get_points(rules)
            );
        }

        return s;
    }
}