pub mod rules;
pub mod schedule_generator;

use crate::match_event;

#[derive(Default, PartialEq)]
enum Type {
    #[default] Null,
    RoundRobin,
    Knockout,
}

#[derive(Default)]
pub struct Stage {
    id: usize,
    name: String,
    teams: Vec<TeamData>,
    match_rules: match_event::Rules,
    round_robin_rules: rules::RoundRobin,
    knockout_rules: rules::Knockout,
    match_schedule: Vec<usize>,
}

// Basics
impl Stage {
    // Build a Stage element.
    pub fn build<S: AsRef<str>>(name: S, teams: Vec<usize>, round_robin_rules: rules::RoundRobin) -> Self {
        let mut stage: Self = Self::default();
        stage.name = String::from(name.as_ref());
        stage.round_robin_rules = round_robin_rules;

        for id in teams {
            stage.teams.push(TeamData {
                team_id: id,
            });
        }

        return stage;
    }

    // Make sure Stage does not have illegal values.
    fn is_valid(&self) -> bool {
        self.id != 0 &&
        self.name != String::default() &&
        self.match_rules.is_valid() &&
        self.get_type() != Type::Null
    }

    // Get the stage type.
    fn get_type(&self) -> Type {
        if self.round_robin_rules.is_valid() {
            if self.knockout_rules == rules::Knockout::default() {
                // Round robin rules are valid, knockout rules are entirely undefined.
                return Type::RoundRobin;
            }
            else {
                // Round robin rules are valid, knockout rules have some assigned values.
                return Type::Null;
            }
        }
        else if self.knockout_rules.is_valid() {
            if self.round_robin_rules == rules::RoundRobin::default() {
                // Knockout rules are valid, round robin rules are entirely undefined.
                return Type::Knockout;
            }
            else {
                // Knockout rules are valid, round robin rules have some assigned values.
                return Type::Null;
            }
        }

        // Neither ruleset is valid.
        return Type::Null;
    }

    // Get the IDs of teams participating in the stage.
    fn get_team_ids(&self) -> Vec<usize> {
        let mut ids: Vec<usize> = Vec::new();
        for team_data in self.teams.iter() {
            ids.push(team_data.team_id);
        }
        return ids;
    }
}

#[derive(Default)]
struct TeamData {
    team_id: usize,
    // do the rest later
}