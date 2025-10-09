// The round robin struct and methods for round-robin stages.
mod match_generator;
mod sorting;

use crate::{types::convert, team::Team, match_event::Game};
use super::{Stage, TeamStageData};

#[derive(Default, Clone, PartialEq, Debug)]
pub enum MatchGenType {
    #[default] Null,
    MatchCount,
    Random,
    Alternating,
}

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
        let mut rr: Self = Self::default();
        rr.rounds = rounds;
        rr.extra_matches = extra_matches;
        rr.points_for_win = points_for_win;
        rr.points_for_ot_win = points_for_ot_win;
        rr.points_for_draw = points_for_draw;
        rr.points_for_ot_loss = points_for_ot_loss;
        rr.points_for_loss = points_for_loss;
        return rr;
    }

    // Make sure the round robin rules do not have illegal values.
    pub fn is_valid(&self) -> bool {
        self.rounds != 0 || self.extra_matches != 0
    }

    // Set up the round-robin stage.
    pub fn setup(&self, stage: &Stage) {
        self.generate_schedule(stage);
    }
}

impl RoundRobin {
    // Get how many matches each team should play.
    pub fn get_theoretical_matches_per_team(&self, stage: &Stage) -> u8 {
        self.get_round_length(stage) * self.rounds
        + self.extra_matches
    }

    // Check if the stage has a valid amount of matches.
    // Increase the match amount if that is not the case.
    fn validate_match_amount(&mut self, stage: &Stage) {
        let matches_per_team: u8 = self.get_theoretical_matches_per_team(stage);

        // Make sure there is at least one match on the stage per team.
        if matches_per_team == 0 {
            self.extra_matches += 1
        }

        // Either the amount of teams or the matches per team must be even.
        if (stage.teams.len() % 2 != 0) && (matches_per_team % 2 != 0) {
            self.extra_matches += 1;
        }
    }

    // Check if the match schedule went according to plan.
    pub fn had_successful_match_generation(&self, stage: &Stage) -> bool {
        self.get_theoretical_total_matches(stage) == convert::usize_to_u16(Game::fetch_stage_matches(stage.id).len())
    }

    // Get how many matches each team has to play to face each team once.
    pub fn get_round_length(&self, stage: &Stage) -> u8 {
        convert::usize_to_u8(stage.teams.len() - 1)
    }

    // Get the teams in the order of the standings.
    fn get_sorted_teams(&self, stage: &Stage) -> Vec<TeamStageData> {
        let mut teams: Vec<TeamStageData> = stage.teams.values().cloned().collect();

        teams.sort_by(|a, b| {
            b.get_points(self).cmp(&a.get_points(self))
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
    pub fn get_matches_per_team(&self, stage: &Stage) -> u8 {
        let matches: f64 = convert::usize_to_f64(Game::fetch_stage_matches(stage.id).len());
        let teams: f64 = convert::usize_to_f64(stage.teams.len());

        return convert::f64_to_u8(matches / teams * 2.0);
    }
}

// Testing.
impl RoundRobin {
    // Get how many matches there should be in the stage in total.
    pub fn get_theoretical_total_matches(&self, stage: &Stage) -> u16 {
        return (self.get_theoretical_matches_per_team(stage) as u16) * convert::usize_to_u16(stage.teams.len()) / 2;
    }

    // Check if the stage has a valid amount of matches.
    // For testing purposes only. For in-game use, see validate_match_amount.
    pub fn has_valid_match_amount(&self, stage: &Stage) -> bool {
        (stage.teams.len() % 2 == 0) || (self.get_theoretical_matches_per_team(stage) % 2 == 0)
    }

    // Get the standings.
    pub fn display_standings(&self, stage: &Stage) -> String {
        let teams: Vec<TeamStageData> = self.get_sorted_teams(stage);

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
                team.get_points(self)
            );
        }

        return s;
    }
}