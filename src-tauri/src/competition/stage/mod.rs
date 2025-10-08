mod schedule_generator;
mod knockout;
pub mod round_robin;

use std::collections::HashMap;
use ::time::Date;

use crate::{
    database::STAGES,
    match_event::{self, Game},
    team::Team,
    time::{date_to_db_string, get_next_annual_date, get_previous_annual_date},
    types::{convert, StageId, TeamId}
};
use self::{round_robin::RoundRobin, knockout::Knockout};

#[derive(Default, Clone, PartialEq)]
enum Type {
    #[default]
    Null,
    RoundRobin,
    Knockout,
}

#[derive(Default, Clone)]
pub struct Stage {
    pub id: StageId,
    name: String,
    num_of_teams: u8,   // How many teams this stage takes. Can be left at 0 for initial stage.
    earliest_date: [u8; 2], // Month and day for the earliest possible match in the Stage.
    latest_date: [u8; 2],   // Month and day for the latest possible match in the Stage.
    pub teams: HashMap<TeamId, TeamData>,
    pub match_rules: match_event::Rules,
    pub round_robin: Option<RoundRobin>,
    knockout: Option<Knockout>,
    stage_type: Type,   // Easy way to check whether the stage is a knockout or round robin type.

    // Tests.
    // pub failures: usize,
}

// Basics
impl Stage {
    // Create an ID.
    fn create_id(&mut self, id: usize) {
        self.id = match id.try_into() {
            Ok(n) => n,
            Err(e) => panic!("{e}")
        };
    }

    // Build a Stage element.
    pub fn build(name: &str, round_robin: Option<RoundRobin>,
    knockout: Option<Knockout>, match_rules: match_event::Rules,
    earliest_date: [u8; 2], latest_date: [u8; 2]) -> Self {
        let mut stage: Self = Self::default();
        stage.name = name.to_string();
        stage.round_robin = round_robin;
        stage.knockout = knockout;
        stage.match_rules = match_rules;
        stage.earliest_date = earliest_date;
        stage.latest_date = latest_date;

        // Set the stage type. Only one of round_robin and knockout can be defined.
        if stage.round_robin.is_some() {
            if stage.knockout.is_none() { stage.stage_type = Type::RoundRobin }
        }
        else if stage.knockout.is_some() { stage.stage_type = Type::Knockout }

        return stage;
    }

    // Build a Stage element and store it in the database. Return the created element.
    pub fn build_and_save(name: &str, round_robin: Option<RoundRobin>,
    knockout: Option<Knockout>, match_rules: match_event::Rules,
    earliest_date: [u8; 2], latest_date: [u8; 2]) -> Self {
        let mut stage: Self = Self::build(name, round_robin, knockout, match_rules, earliest_date, latest_date);
        stage.create_id(STAGES.lock().unwrap().len() + 1);
        stage.save();
        return stage;
    }

    pub fn fetch_from_db(id: &StageId) -> Self {
        STAGES.lock().unwrap().get(id)
            .expect(&format!("no Stage with id {id:#?}")).clone()
    }

    // Update the Stage to database.
    pub fn save(&self) {
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

    // Get the IDs of teams participating in the stage.
    fn get_team_ids(&self) -> Vec<TeamId> {
        let mut ids: Vec<TeamId> = Vec::new();
        for team_data in self.teams.values() {
            ids.push(team_data.team_id);
        }
        return ids;
    }

    // Get the previous earliest match date as a date object.
    pub fn get_previous_start_date(&self) -> Date {
        get_previous_annual_date(self.earliest_date[0], self.earliest_date[1])
    }

    // Get the previous latest match date as a date object.
    fn get_previous_end_date(&self) -> Date {
        get_previous_annual_date(self.latest_date[0], self.latest_date[1])
    }

    // Get the next earliest match date as a date object.
    fn get_next_start_date(&self) -> Date {
        get_next_annual_date(self.earliest_date[0], self.earliest_date[1])
    }

    // Get the next latest match date as a date object.
    pub fn get_next_end_date(&self) -> Date {
        get_next_annual_date(self.latest_date[0], self.latest_date[1])
    }

}

// Functional.
impl Stage {
    // Add teams to this stage.
    pub fn add_teams(&mut self, team_ids: &Vec<TeamId>) {
        for id in team_ids.iter() {
            self.teams.insert(*id, TeamData::build(*id));
        }
    }

    // Set up the stage so the competition can use it, and save to database.
    pub fn setup(&mut self, team_ids: &Vec<TeamId>) {
        self.add_teams(team_ids);
        self.round_robin.as_ref().unwrap().generate_schedule(self);   // todo: make sure knockouts do not do this.
        self.save();
    }
}

// Testing
impl Stage {
    // Get a nice printed display of match schedule.
    pub fn display_match_schedule(&self) -> String {
        let mut s: String = String::new();
        let games: Vec<(Date, Vec<Game>)> = Game::fetch_stage_matches_by_date(self.id);

        for date in games.iter() {
            if s.len() > 0 {
                s += "\n\n";
            }
            s += &date_to_db_string(&date.0);

            for game in date.1.iter() {
                s += &format!("\n{}", game.get_name_and_score_if_started());
            }
        }

        return s;
    }
}

#[derive(Default, Clone)]
pub struct TeamData {
    pub team_id: TeamId,
    pub regular_wins: u8,
    pub ot_wins: u8,
    pub draws: u8,
    pub ot_losses: u8,
    pub regular_losses: u8,
    pub goals_scored: u16,
    pub goals_conceded: u16,
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
    fn get_points(&self, rr: &RoundRobin) -> u8 {
        self.regular_wins * rr.points_for_win +
        self.ot_wins * rr.points_for_ot_win +
        self.draws * rr.points_for_draw +
        self.ot_losses * rr.points_for_ot_loss +
        self.regular_losses * rr.points_for_loss
    }

    fn get_goal_difference(&self) -> i8 {
        let gf: i16 = convert::u16_to_i16(self.goals_scored);
        let ga: i16 = convert::u16_to_i16(self.goals_conceded);
        return convert::i16_to_i8(gf - ga);
    }
}