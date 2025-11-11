// The round robin struct and methods for round-robin stages.
mod schedule_validator;

use crate::{competition::{Competition, season::{Season, team::TeamCompData}}, match_event::Game, team::Team, types::{convert}};

#[derive(Default, Clone, PartialEq, Debug)]
pub enum MatchGenType {
    #[default] Null,
    MatchCount,
    Random,
    Alternating,
}

#[derive(Debug, serde::Serialize)]
#[derive(Default, Clone)]
pub struct RoundRobin {
    pub rounds: u8, // How many times each team plays one another.
    pub extra_matches: u8,  // How many matches should be scheduled in addition to rounds.
    pub points_for_win: u8,
    pub points_for_ot_win: u8,
    pub points_for_draw: u8,
    pub points_for_ot_loss: u8,
    pub points_for_loss: u8,
}

// Basics
impl RoundRobin {
    pub const MATCH_GEN_TYPE: MatchGenType = MatchGenType::Alternating;

    pub fn build(rounds: u8, extra_matches: u8, points_for_win: u8, points_for_ot_win: u8, points_for_draw: u8, points_for_ot_loss: u8, points_for_loss: u8) -> Self {
        Self {
            rounds: rounds,
            extra_matches: extra_matches,
            points_for_win: points_for_win,
            points_for_ot_win: points_for_ot_win,
            points_for_draw: points_for_draw,
            points_for_ot_loss: points_for_ot_loss,
            points_for_loss: points_for_loss,
        }
    }

    // Make sure the round robin rules do not have illegal values.
    pub fn is_valid(&self) -> bool {
        self.rounds != 0 || self.extra_matches != 0
    }
}

impl RoundRobin {
    // Get how many matches each team should play.
    pub fn get_theoretical_matches_per_team(&self, season: &Season) -> u8 {
        self.get_round_length(season) * self.rounds
        + self.extra_matches
    }

    // Check if the stage has a valid amount of matches.
    // Increase the match amount if that is not the case.
    fn validate_match_amount(&mut self, season: &Season) {
        let matches_per_team = self.get_theoretical_matches_per_team(season);

        // Make sure there is at least one match on the stage per team.
        if matches_per_team == 0 {
            self.extra_matches += 1
        }

        // Either the amount of teams or the matches per team must be even.
        if (season.teams.len() % 2 != 0) && (matches_per_team % 2 != 0) {
            self.extra_matches += 1;
        }
    }

    // Check if the match schedule went according to plan.
    pub fn had_successful_match_generation(&self, season: &Season) -> bool {
        self.get_theoretical_total_matches(season) == convert::int::<usize, u16>(season.get_all_games().len())
    }

    // Get how many matches each team has to play to face each team once.
    pub fn get_round_length(&self, season: &Season) -> u8 {
        convert::int::<usize, u8>(season.teams.len() - 1)
    }
}

// Testing.
impl RoundRobin {
    // Get how many matches there should be in the stage in total.
    pub fn get_theoretical_total_matches(&self, season: &Season) -> u16 {
        return (self.get_theoretical_matches_per_team(season) as u16) * convert::int::<usize, u16>(season.teams.len()) / 2;
    }

    // Check if the stage has a valid amount of matches.
    // For testing purposes only. For in-game use, see validate_match_amount.
    pub fn has_valid_match_amount(&self, season: &Season) -> bool {
        (season.teams.len() % 2 == 0) || (self.get_theoretical_matches_per_team(season) % 2 == 0)
    }
}