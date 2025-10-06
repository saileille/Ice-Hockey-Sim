pub mod event;
pub mod team;

use self::team::TeamData;
use self::event::Shot;

use crate::custom_types::{GameId, StageId, TeamId};
use crate::database::GAMES;

use crate::event as logic_event;
use crate::competition::stage::Stage;

#[derive(Default, Clone)]
pub struct Game {
    pub id: GameId,
    home: TeamData,
    away: TeamData,
    gameclock: Clock,
    stage_id: StageId,
    attacker: TeamData,
    defender: TeamData,
}

impl Game { // Basics.
    pub fn build(home: TeamId, away: TeamId, stage: StageId) -> Self {
        let mut game = Game::default();

        game.home = TeamData::new(home);
        game.away = TeamData::new(away);
        game.gameclock = Clock::default();
        game.stage_id = stage;

        return game;
    }
    
    pub fn build_and_save(home: TeamId, away: TeamId, stage: StageId) -> Self {
        let mut game: Self = Self::build(home, away, stage);
        game.id = GAMES.lock().unwrap().len() as GameId;
        game.update_to_db();
        return game;
    }

    pub fn fetch_from_db(id: &GameId) -> Self {
        GAMES.lock().unwrap().get(id)
            .expect(&format!("no Game with id {id:#?}")).clone()
    }

    // Update the Team to database.
    pub fn update_to_db(&self) {
        GAMES.lock()
            .expect(&format!("something went wrong when trying to update Game {}: {} to GAMES", self.id, self.get_name()))
            .insert(self.id, self.clone());
    }

    // Delete the Team from the database.
    pub fn delete_from_db(&self) {
        GAMES.lock()
            .expect(&format!("something went wrong when trying to delete Game {}: {} from GAMES", self.id, self.get_name()))
            .remove(&self.id);
    }

    // Make sure the game does not contain illegal values.
    fn is_valid(&self) -> bool {
        self.home.is_valid() && self.away.is_valid() && self.get_rules().is_valid()
    }

    // Get the game rules.
    fn get_rules(&self) -> Rules {
        Stage::fetch_from_db(&self.stage_id).match_rules
    }
}

impl Game {
    // Call when both teams must submit their lineups.
    fn get_team_lineups(&mut self) {
        self.home.lineup = self.home.get_team_clone().lineup;
        self.away.lineup = self.away.get_team_clone().lineup;
    }

    // Do things like submitting lineups.
    fn do_pre_game_tasks(&mut self) {
        self.get_team_lineups();
    }

    // Do everything that needs to be done after the game is concluded.
    fn do_post_game_tasks(&mut self) {
        self.attacker = TeamData::default();
        self.defender = TeamData::default();
    }

    // Play the game.
    pub fn play(&mut self) {
        self.do_pre_game_tasks();
        self.simulate();    // The actual game is played here.
        self.do_post_game_tasks();
    }

    // Simulate a game of ice hockey.
    fn simulate(&mut self) {
        while self.gameclock.periods_completed < self.get_rules().periods {
            self.simulate_period();
            self.gameclock.reset_period_time();
            self.gameclock.periods_completed += 1;
        }

        // To save space, let's clear all duplicate data.
        self.attacker = TeamData::default();
        self.defender = TeamData::default();
    }

    // Simulate a period of ice hockey.
    fn simulate_period(&mut self) {
        while (self.gameclock.period_total_seconds as u16) < self.get_rules().period_length {
            self.simulate_second();
            self.gameclock.advance();
        }
    }

    // Simulate a second of ice hockey.
    fn simulate_second(&mut self) {
        self.change_players_on_ice();
        self.change_puck_possession();
        self.attempt_shot();
        self.update_team_data();
    }

    // Change the players on ice for home and away teams.
    fn change_players_on_ice(&mut self) {
        self.home.change_players_on_ice();
        self.away.change_players_on_ice();
    }

    // Change which team has the puck.
    fn change_puck_possession(&mut self) {
        let modifier: f64 = self.home.players_on_ice.get_player_clones().get_skaters_ability_ratio(self.away.players_on_ice.get_player_clones());
        if logic_event::Type::fetch_from_db(&logic_event::Id::PuckPossessionChange).get_outcome(modifier) {
            self.attacker = self.home.clone();
            self.defender = self.away.clone();
        }
        else {
            self.attacker = self.away.clone();
            self.defender = self.home.clone();
        }
    }

    // The attacking team attempts to shoot the puck.
    fn attempt_shot(&mut self) {
        let modifier: f64 = self.attacker.players_on_ice.get_player_clones().get_skaters_ability_ratio(self.defender.players_on_ice.get_player_clones());
        let success: bool = logic_event::Type::fetch_from_db(&logic_event::Id::ShotAtGoal).get_outcome(modifier);

        if success {
            let mut shot: Shot = Shot::new(self.gameclock.clone(), self.attacker.players_on_ice.clone(), self.defender.players_on_ice.clone());
            shot.create_shooter_and_assisters();
            shot.calculate_goal();
            self.attacker.shots.push(shot);
        }
    }

    // Update the home and away team data with what attacker and defender have done.
    fn update_team_data(&mut self) {
        if self.home.team_id == self.attacker.team_id {
            self.home = self.attacker.clone();
            self.away = self.defender.clone();
        }
        else {
            self.home = self.defender.clone();
            self.away = self.attacker.clone();
        }
    }

    // Get the home and away team names.
    fn get_name(&self) -> String {
        format!("{} - {}", self.home.get_team_clone().name, self.away.get_team_clone().name)
    }

    // Get the home and away team names, as well as the game score.
    pub fn get_name_and_score(&self) -> String {
        format!("{} {} - {} {}", self.home.get_team_clone().name, self.home.get_goal_amount(), self.away.get_goal_amount(), self.away.get_team_clone().name)
    }

    // Reset the match by setting the time and team data back to default values.
    fn reset(&mut self) {
        self.gameclock.reset();
        self.home.reset();
        self.away.reset();
    }
}

impl Game { // Methods for testing phase.
    // Generate an ascetic infodump about which team scored and when.
    pub fn get_simple_boxscore(&self) -> String {
        struct BoxscoreGoal {
            time: Clock,
            team: String,
            score_info: String,
        }

        let home_name: String = self.home.get_team_clone().name;
        let away_name: String = self.away.get_team_clone().name;

        let mut events: Vec<BoxscoreGoal> = Vec::new();
        for goal in self.home.shots.iter() {
            if goal.is_goal {
                events.push(BoxscoreGoal {
                    time: goal.event.time.clone(),
                    team: home_name.clone(),
                    score_info: goal.scorer_and_assists_to_string(),
                });
            }
        }

        for goal in self.away.shots.iter() {
            if goal.is_goal {
                events.push(BoxscoreGoal {
                    time: goal.event.time.clone(),
                    team: away_name.clone(),
                    score_info: goal.scorer_and_assists_to_string(),
                });
            }
        }

        events.sort_by(
            |a: &BoxscoreGoal, b: &BoxscoreGoal| 
            a.time.get_game_total_seconds(self.get_rules().period_length)
            .cmp(&b.time.get_game_total_seconds(self.get_rules().period_length))
        );

        let mut boxscore: String = String::new();
        let mut home_goal_counter: u16 = 0;
        let mut away_goal_counter: u16 = 0;
        for event in events.iter() {
            if event.team == home_name {
                home_goal_counter += 1;
            }
            else {
                away_goal_counter += 1;
            }

            boxscore += &format!("{}: {} - {} {} - {} \n", event.time.game_time_to_string(self.get_rules().period_length), home_goal_counter, away_goal_counter, event.team, event.score_info);
        }

        return boxscore;
    }
}

#[derive(Default, Clone)]
pub struct Rules {
    periods: u8,
    period_length: u16,
    overtime_length: u16,
    continuous_overtime: bool,
}

// Basics.
impl Rules {
    pub fn build(periods: u8, period_length: u16, overtime_length: u16, continous_overtime: bool) -> Self {
        Rules {
            periods: periods,
            period_length: period_length,
            overtime_length: overtime_length,
            continuous_overtime: continous_overtime,
        }
    }

    // Make sure the rules do not contain illegal values.
    pub fn is_valid(&self) -> bool {
        self.periods != 0 && self.period_length != 0
    }
}

// Functional.
impl Rules {
    // Get the total regular time of the game in seconds.
    fn get_regular_time(&self) -> u16 {
        (self.periods as u16) * self.period_length
    }
}

#[derive(Default, Clone)]
struct Clock {
    periods_completed: u8,
    period_total_seconds: u16,
}

// Basics.
impl Clock {
    fn build(periods_completed: u8, seconds: u16) -> Self {
        Clock {
            periods_completed: periods_completed,
            period_total_seconds: seconds,
        }
    }
}

// Functional.
impl Clock {
    // Check if the game time is over.
    fn is_game_time_over(&self, rules: &Rules) -> bool {
        if rules.continuous_overtime ||
        !self.is_regular_time_over(rules) { 
            return false;
        }


        return true;
    }

    // Check if the regular time of the game is over.
    fn is_regular_time_over(&self, rules: &Rules) -> bool {
        self.periods_completed >= rules.periods
    }

    // Check if the overtime is over.
    fn is_overtime_over(&self, rules: &Rules) -> bool {
        if rules.continuous_overtime { return false; }

        let overtime_length: i32 = (self.get_game_total_seconds(rules.period_length) as i32) - (rules.get_regular_time() as i32);
        return overtime_length >= (rules.overtime_length as i32);
    }

    fn time_to_string(seconds: u32) -> String {
        format!("{}:{:0>2}", seconds / 60, seconds % 60)
    }

    // Get a minutes-seconds representation of the time that has passed in the game.
    fn game_time_to_string(&self, period_length: u16) -> String {
        Clock::time_to_string(self.get_game_total_seconds(period_length))
    }

    // Get a minutes-seconds representation of the time that has passed in the period.
    fn period_time_to_string(&self) -> String {
        Clock::time_to_string(self.period_total_seconds as u32)
    }

    // Get the total seconds that have passed in the game.
    fn get_game_total_seconds(&self, period_length: u16) -> u32 {
        (self.periods_completed as u32) * (period_length as u32) + (self.period_total_seconds as u32)
    }
    
    // Get the amount of minutes that have passed in the period.
    fn get_period_minutes(&self) -> u8 {
        (self.period_total_seconds / 60) as u8
    }

    // Get the amount of seconds that have passed in the period, after full minutes have been taken out.
    fn get_period_seconds(&self) -> u8 {
        (self.period_total_seconds % 60) as u8
    }

    // Get the amount of minutes that have passed in the game.
    fn get_game_minutes(&self, period_length: u16) -> u16 {
        (self.get_game_total_seconds(period_length) / 60) as u16
    }

    // Get the amount of seconds that have passed in the game, after full minutes have been taken out.
    fn get_game_seconds(&self, period_length: u16) -> u8 {
        (self.get_game_total_seconds(period_length) % 60) as u8
    }

    // Advance time by one second.
    fn advance(&mut self) {
        self.period_total_seconds += 1;
    }

    // Reset the period clock.
    fn reset_period_time(&mut self) {
        self.period_total_seconds = 0;
    }

    // Reset the clock completely.
    fn reset(&mut self) {
        self.period_total_seconds = 0;
        self.periods_completed = 0;
    }
}