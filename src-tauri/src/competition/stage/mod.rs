pub mod rules;
pub mod schedule_generator;

use crate::custom_types::{StageId, GameId, TeamId};
use crate::database::STAGES;
use crate::match_event;
use crate::team::Team;

#[derive(Default, Clone, PartialEq)]
enum Type {
    #[default] Null,
    RoundRobin,
    Knockout,
}

#[derive(Default, Clone)]
pub struct Stage {
    pub id: StageId,
    name: String,
    pub teams: Vec<TeamData>,
    pub match_rules: match_event::Rules,
    round_robin_rules: Option<rules::RoundRobin>,
    knockout_rules: Option<rules::Knockout>,
    stage_type: Type,   // Easy way to check whether the stage is a knockout or round robin type.
    schedule: Vec<GameId>,

    // Tests.
    matchday_tests: Vec<Vec<GameId>>,
    pub failures: usize,
}

// Basics
impl Stage {
    // Build a Stage element.
    pub fn build<S: AsRef<str>>(name: S, round_robin_rules: Option<rules::RoundRobin>,
    knockout_rules: Option<rules::Knockout>, match_rules: match_event::Rules) -> Self {
        let mut stage: Self = Self::default();
        stage.name = String::from(name.as_ref());
        stage.round_robin_rules = round_robin_rules;
        stage.knockout_rules = knockout_rules;
        stage.match_rules = match_rules;

        // Set the stage type. Only one of round_robin_rules and knockout_rules can be defined.
        if stage.round_robin_rules.is_some() {
            if stage.knockout_rules.is_none() { stage.stage_type = Type::RoundRobin }
        }
        else if stage.knockout_rules.is_some() { stage.stage_type = Type::Knockout }

        return stage;
    }

    // Build a Stage element and store it in the database. Return the created element.
    pub fn build_and_save<S: AsRef<str>>(name: S, round_robin_rules: Option<rules::RoundRobin>,
    knockout_rules: Option<rules::Knockout>, match_rules: match_event::Rules) -> Self {
        let mut stage: Self = Self::build(name, round_robin_rules, knockout_rules, match_rules);
        stage.id = (STAGES.lock().unwrap().len() + 1) as StageId;
        stage.update_to_db();
        return stage;
    }

    pub fn fetch_from_db(id: &StageId) -> Self {
        STAGES.lock().unwrap().get(id)
            .expect(&format!("no Stage with id {id:#?}")).clone()
    }

    // Update the Stage to database.
    pub fn update_to_db(&self) {
        STAGES.lock()
            .expect(&format!("something went wrong when trying to update Stage {}: {} to STAGES", self.id, self.name))
            .insert(self.id, self.clone());
    }

    // Make sure Stage does not have illegal values.
    fn is_valid(&self) -> bool {
        self.id != 0 &&
        self.name != String::default() &&
        self.teams.len() > 1 &&
        self.match_rules.is_valid() &&
        self.stage_type != Type::Null
    }

    // Add teams to this stage.
    fn add_teams(&mut self, team_ids: Vec<TeamId>) {
        for id in team_ids {
            self.teams.push(TeamData::build(id));
        }
    }

    // Get the IDs of teams participating in the stage.
    fn get_team_ids(&self) -> Vec<TeamId> {
        let mut ids: Vec<TeamId> = Vec::new();
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
        let matches: f64 = self.schedule.len() as f64;
        let teams: f64 = self.teams.len() as f64;

        return (matches / teams * 2.0) as u8;
    }

    // Get how many matches each team should play.
    // For round robins only.
    fn get_theoretical_matches_per_team(&self) -> u8 {
        let rr: &rules::RoundRobin = self.round_robin_rules.as_ref().unwrap();
        self.get_round_length() * rr.rounds
        + rr.extra_matches
    }

    // Get how many matches each team has to play to face each team once.
    fn get_round_length(&self) -> u8 {
        (self.teams.len() as u8) - 1
    }

    // Check if the stage has a valid amount of matches.
    // Increase the matches by one if that is not the case.
    // For round robins only.
    fn validate_match_amount(&mut self) {
        let mut rr: rules::RoundRobin = self.round_robin_rules.as_ref().unwrap().clone();
        let matches_per_team: u8 = self.get_theoretical_matches_per_team();

        // Make sure there is at least one match on the stage per team.
        if matches_per_team == 0 {
            rr.extra_matches += 1
        }

        // Either the amount of teams or the matches per team must be even.
        if (self.teams.len() % 2 != 0) && (matches_per_team % 2 != 0) {
            rr.extra_matches += 1;
        }

        self.round_robin_rules = Some(rr);
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
        self.get_theoretical_total_matches() == (self.schedule.len() as u16)
    }
}

#[derive(Default, Clone)]
pub struct TeamData {
    pub team_id: TeamId,
    regular_wins: u8,
    ot_wins: u8,
    draws: u8,
    ot_losses: u8,
    regular_losses: u8,
    goals_scored: u8,
    goals_conceded: u8,
}

// Basics.
impl TeamData {
    fn build(team_id: TeamId) -> Self {
        let mut teamdata: Self = Self::default();
        teamdata.team_id = team_id;
        return teamdata;
    }

    // Get the team element tied to this TeamData.
    fn get_team(&self) -> Team {
        Team::fetch_from_db(&self.team_id)
    }
}

// Functional
impl TeamData {
    fn get_game_count(&self) -> u8 {
        self.get_wins() + self.get_losses() + self.draws
    }

    fn get_wins(&self) -> u8 {
        self.regular_wins + self.ot_wins
    }

    fn get_losses(&self) -> u8 {
        self.regular_losses + self.ot_losses
    }

    // Get points accumulated in a round robin stage.
    fn get_points(&self, rules: &rules::RoundRobin) -> u8 {
        self.regular_wins * rules.points_for_win +
        self.ot_wins * rules.points_for_ot_win +
        self.draws * rules.points_for_draw +
        self.ot_losses * rules.points_for_ot_loss +
        self.regular_losses * rules.points_for_loss
    }

    fn get_goal_difference(&self) -> i8 {
        (self.goals_scored as i8) - (self.goals_conceded as i8)
    }
}