// Data for teams.

use ordinal::ToOrdinal;
use serde_json::json;

use crate::{competition::{format, Competition}, match_event::team::TeamGameData, team::Team, types::{convert, TeamId}};

#[derive(Debug, serde::Serialize)]
#[derive(PartialEq)]
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
    pub fn get_team(&self) -> Team {
        Team::fetch_from_db(&self.team_id)
    }

    // Get relevant information for a competition screen.
    pub fn get_json(&self, comp: &Competition, index: usize) -> serde_json::Value {
        json!({
            "id": self.team_id,
            "name": self.get_team().name,
            "rank": (index + 1).to_ordinal_string(),
            "games": self.get_game_count(),
            "wins": self.regular_wins,
            "ot_wins": self.ot_wins,
            "draws": self.draws,
            "ot_losses": self.ot_losses,
            "losses": self.regular_losses,
            "total_wins": self.get_wins(),
            "total_losses": self.get_losses(),
            "goals_scored": self.goals_scored,
            "goals_conceded": self.goals_conceded,
            "goal_difference": self.get_goal_difference(),
            "points": self.get_points(&comp.get_round_robin_format()),
            "seed": self.seed
        })
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

    pub fn get_losses(&self) -> u8 {
        self.regular_losses + self.ot_losses
    }

    // Get points accumulated in a round robin stage.
    pub fn get_points(&self, rr_option: &Option<format::round_robin::RoundRobin>) -> u8 {
        if rr_option.is_none() { return 0; }
        let rr = rr_option.as_ref().unwrap();

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