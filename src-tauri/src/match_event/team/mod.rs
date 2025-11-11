pub mod cache;

use serde_json::json;

use crate::{competition::season::team::TeamCompData, match_event::event::Shot, team::{Team, lineup::LineUp}, types::{TeamId, convert}};

#[derive(Debug)]
#[derive(Default, Clone)]
pub struct TeamGameData {
    pub team_id: TeamId,
    pub team_seed: u8,
    pub shots: Vec<Shot>,
    pub lineup: LineUp,
    penalties: Vec<String>, // Placeholder.
}

impl TeamGameData { // Basics.
    pub fn build(team: &TeamCompData) -> Self {
        Self {
            team_id: team.team_id,
            team_seed: team.seed,
            ..Default::default()
        }
    }

    // Make sure the TeamData does not contain illegal values.
    pub fn is_valid(&self) -> bool {
        self.team_id != 0
    }

    // Get a clone of the team.
    pub fn get_team(&self) -> Team {
        Team::fetch_from_db(&self.team_id)
    }

    pub fn get_comp_screen_json(&self) -> serde_json::Value {
        json!({
            "id": self.team_id,
            "name": self.get_team().name,
            "seed": self.team_seed,
            "goals": self.get_goal_amount()
        })
    }
}

// Functional.
impl TeamGameData {
    fn get_shot_amount(&self) -> u16 {
        convert::int::<usize, u16>(self.shots.len())
    }

    pub fn get_goal_amount(&self) -> u16 {
        let mut goal_counter = 0;
        for shot in self.shots.iter() {
            if shot.is_goal { goal_counter += 1; }
        }
        return goal_counter;
    }
}