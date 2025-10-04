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
    pub teams: Vec<TeamData>,
    match_rules: match_event::Rules,
    round_robin_rules: rules::RoundRobin,
    knockout_rules: rules::Knockout,
    // match_schedule: Vec<usize>,

    // Testing stuff.
    pub match_tests: Vec<[usize; 2]>,
    pub failures: usize,
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
        self.teams.len() > 1 &&
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

// Functional.
impl Stage {
    // Get the amount of actual games each team plays.
    pub fn get_matches_per_team(&self) -> u8 {
        let matches: f64 = self.match_tests.len() as f64;
        let teams: f64 = self.teams.len() as f64;
        
        return (matches / teams * 2.0) as u8;
    }

    // Get how many matches each team should play.
    // For round robins only.
    fn get_theoretical_matches_per_team(&self) -> u8 {
        self.get_round_length() * self.round_robin_rules.rounds
        + self.round_robin_rules.extra_matches
    }

    // Get how many matches each team has to play to face each team once.
    fn get_round_length(&self) -> u8 {
        (self.teams.len() as u8) - 1
    }

    // Check if the stage has a valid amount of matches.
    // Increase the matches by one if that is not the case.
    // For round robins only.
    fn validate_match_amount(&mut self) {
        let matches_per_team: u8 = self.get_theoretical_matches_per_team();
        
        // Make sure there will be some matches on the stage.
        if matches_per_team == 0 {
            self.round_robin_rules.extra_matches += 1
        }

        // Either the amount of teams or the matches per team must be even.
        if (self.teams.len() % 2 != 0) && (matches_per_team % 2 != 0) {
            self.round_robin_rules.extra_matches += 1;
        }
    }
}

// Testing
impl Stage {
    // Get how many matches there should be in the stage in total.
    // For round robins only.
    pub fn get_theoretical_total_matches(&self) -> u16 {
        (self.get_theoretical_matches_per_team() as u16) * (self.teams.len() as u16) / 2
    }

    // Check if the stage has a valid amount of matches.
    // For testing purposes only. For in-game use, see validate_match_amount.
    pub fn has_valid_match_amount(&self) -> bool {
        (self.teams.len() % 2 == 0) || (self.get_theoretical_matches_per_team() % 2 == 0)
    }

    // Check if the match schedule went according to plan.
    pub fn had_successful_match_generation(&self) -> bool {
        self.get_theoretical_total_matches() == (self.match_tests.len() as u16)
    }
}

#[derive(Default)]
pub struct TeamData {
    pub team_id: usize,
    // do the rest later
}