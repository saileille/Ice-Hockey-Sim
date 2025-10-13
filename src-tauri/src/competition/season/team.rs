// Data for teams.

use crate::{competition::format, match_event::team::TeamGameData, team::Team, types::{convert, TeamId}};

#[derive(Debug)]
#[derive(Default, Clone)]
pub struct TeamCompData {
    pub team_id: TeamId,

    // Seed is mostly used in knockouts, but can be used for tie-breakers in round-robin as well.
    // The lower the value, the better the seed is.
    // 0 can theoretically be used, but for clarity, maybe use it only when every team's seed is 0?
    pub seed: u8,
    pub regular_wins: u8,
    pub ot_wins: u8,
    pub draws: u8,
    pub ot_losses: u8,
    pub regular_losses: u8,
    pub goals_scored: u16,
    pub goals_conceded: u16,
}

// Basics.
impl TeamCompData {
    pub fn build(team_id: TeamId, seed: u8) -> Self {
        let mut teamdata = Self::default();
        teamdata.team_id = team_id;
        teamdata.seed = seed;
        return teamdata;
    }

    // Get the team element tied to this TeamData.
    fn get_team(&self) -> Team {
        Team::fetch_from_db(&self.team_id)
    }
}

// Functional
impl TeamCompData {
    pub fn get_game_count(&self) -> u8 {
        self.get_wins() + self.get_losses() + self.draws
    }

    pub fn get_wins(&self) -> u8 {
        self.regular_wins + self.ot_wins
    }

    fn get_losses(&self) -> u8 {
        self.regular_losses + self.ot_losses
    }

    // Get points accumulated in a round robin stage.
    pub fn get_points(&self, rr_option: &Option<&format::round_robin::RoundRobin>) -> u8 {
        if rr_option.is_none() { return 0; }
        let rr = rr_option.unwrap();

        self.regular_wins * rr.points_for_win +
        self.ot_wins * rr.points_for_ot_win +
        self.draws * rr.points_for_draw +
        self.ot_losses * rr.points_for_ot_loss +
        self.regular_losses * rr.points_for_loss
    }

    pub fn get_goal_difference(&self) -> i8 {
        let gf = convert::u16_to_i16(self.goals_scored);
        let ga = convert::u16_to_i16(self.goals_conceded);
        return convert::i16_to_i8(gf - ga);
    }

    // Update the team data after a match.
    pub fn update(&mut self, this: &TeamGameData, opponent: &TeamGameData, had_overtime: bool) {
        let self_goals = this.get_goal_amount();
        let opp_goals = opponent.get_goal_amount();

        // This team won.
        if self_goals > opp_goals {
            if !had_overtime { self.regular_wins += 1; }
            else { self.ot_wins += 1; }
        }
        else if self_goals < opp_goals {
            if !had_overtime { self.regular_losses += 1; }
            else { self.ot_losses += 1; }
        }
        else { self.draws += 1; }

        self.goals_scored += self_goals;
        self.goals_conceded += opp_goals;
    }
}