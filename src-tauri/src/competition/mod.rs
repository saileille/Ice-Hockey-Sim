// Competition data.
pub mod stage;

use crate::types::{CompetitionId, StageId, TeamId};
use crate::database::COMPETITIONS;

use crate::team::Team;
use self::stage::Stage;

#[derive(Default, Clone)]
pub struct Competition {
    id: CompetitionId,
    name: String,
    team_ids: Vec<TeamId>,
    stage_ids: Vec<Vec<StageId>>,
}

// Basics.
impl Competition {
    // Create a new ID.
    fn create_id(&mut self, id: usize) {
        self.id = match id.try_into() {
            Ok(id) => id,
            Err(e) => panic!("{e}")
        };
    }

    // Build a Competition element.
    pub fn build<S: AsRef<str>>(name: S, teams: Vec<Team>, stages: Vec<Vec<Stage>>) -> Self {
        let mut comp: Self = Self::default();
        comp.name = String::from(name.as_ref());

        // NOTE: build_and_save has already saved these to the database.
        for team in teams {
            comp.team_ids.push(team.id);
        }

        // NOTE: build_and_save has already saved these to the database.
        for comp_phase in stages {
            let mut comp_phase_ids: Vec<StageId> = Vec::new();
            for stage in comp_phase {
                comp_phase_ids.push(stage.id);
            }
            comp.stage_ids.push(comp_phase_ids);
        }

        return comp;
    }

    // Build a Competition element and store it in the database. Return the created element.
    pub fn build_and_save<S: AsRef<str>>(name: S, teams: Vec<Team>, stages: Vec<Vec<Stage>>) -> Self {
        let mut comp: Self = Self::build(name, teams, stages);
        comp.create_id(COMPETITIONS.lock().unwrap().len() + 1);
        comp.update_to_db();
        return comp;
    }

    pub fn fetch_from_db(id: &CompetitionId) -> Self {
        COMPETITIONS.lock().unwrap().get(id)
            .expect(&format!("no Competition with id {id:#?}")).clone()
    }

    // Update the Stage to database.
    pub fn update_to_db(&self) {
        COMPETITIONS.lock()
            .expect(&format!("something went wrong when trying to update Competition {}: {} to COMPETITIONS", self.id, self.name))
            .insert(self.id, self.clone());
    }
}