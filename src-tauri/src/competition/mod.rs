// Competition data.
pub mod stage;

use crate::{
    types::{
        CompetitionId,
        StageId,
        TeamId
    },
    database::{COMPETITIONS},
    team::Team
};
use self::stage::Stage;

#[derive(Default, Clone)]
pub struct Competition {
    id: CompetitionId,
    name: String,
    team_ids: Vec<TeamId>,
    pub stage_ids: Vec<StageId>,
}

// Basics.
impl Competition {
    // Create a new ID.
    fn create_id(&mut self, id: usize) {
        self.id = match id.try_into() {
            Ok(n) => n,
            Err(e) => panic!("{e}")
        };
    }

    // Build a Competition element.
    pub fn build(name: &str, teams: Vec<Team>, stages: Vec<Stage>) -> Self {
        let mut comp: Self = Self::default();
        comp.name = name.to_string();

        // NOTE: build_and_save has already saved these to the database.
        for team in teams {
            comp.team_ids.push(team.id);
        }

        // NOTE: build_and_save has already saved these to the database.
        for stage in stages {
            comp.stage_ids.push(stage.id);
        }

        return comp;
    }

    // Build a Competition element and store it in the database. Return the created element.
    pub fn build_and_save(name: &str, teams: Vec<Team>, stages: Vec<Stage>) -> Self {
        let mut comp: Self = Self::build(name, teams, stages);
        comp.create_id(COMPETITIONS.lock().unwrap().len() + 1);
        comp.save();
        return comp;
    }

    pub fn fetch_from_db(id: &CompetitionId) -> Self {
        COMPETITIONS.lock().unwrap().get(id)
            .expect(&format!("no Competition with id {id:#?}")).clone()
    }

    // Update the Stage to database.
    pub fn save(&self) {
        COMPETITIONS.lock()
            .expect(&format!("something went wrong when trying to update Competition {}: {} to COMPETITIONS", self.id, self.name))
            .insert(self.id, self.clone());
    }
}

// Functional.
impl Competition {
    // Set up the competition for a new season.
    pub fn setup(&self) {
        self.setup_teams();
        let mut initial_stage: Stage = Stage::fetch_from_db(&self.stage_ids[0]);
        initial_stage.setup(&self.team_ids);
    }
}

// Testing.
impl Competition {
    // Set up all teams in the competition.
    fn setup_teams(&self) {
        for id in self.team_ids.iter() {
            Team::fetch_from_db(id).setup(0, 0);
        }
    }
}