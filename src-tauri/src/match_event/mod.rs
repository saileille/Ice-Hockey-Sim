mod event;
mod team;

use std::collections::HashMap;
use time::Date;

use crate::{
    competition::stage::{
        Stage,
        TeamData as StageTeamData,
    },
    team::Team,
    database::{GAMES},
    event as logic_event,
    time::db_string_to_date, types::{
        convert, GameId, StageId, TeamId
    }
};
use self::{
    team::TeamData,
    event::Shot
};

#[derive(Default, Clone)]
pub struct Game {
    pub date: String,
    pub id: GameId,
    home: TeamData,
    away: TeamData,
    clock: Clock,
    stage_id: StageId,
    attacker: Option<TeamData>,
    defender: Option<TeamData>,
}

impl Game { // Basics.
    // Create an ID.
    fn create_id(&mut self, id: usize) {
        self.id = match id.try_into() {
            Ok(n) => n,
            Err(e) => panic!("{e}")
        };
    }

    pub fn build(home: TeamId, away: TeamId, stage: StageId) -> Self {
        let mut game: Game = Game::default();

        game.home = TeamData::build(home);
        game.away = TeamData::build(away);
        game.clock = Clock::default();
        game.stage_id = stage;

        return game;
    }

    pub fn build_and_save(home: TeamId, away: TeamId, stage: StageId, date: &str) -> Self {
        let mut game: Self = Self::build(home, away, stage);
        game.date = date.to_string();
        game.save();

        return game;
    }

    // Get all matches that are part of the specific stage.
    pub fn fetch_stage_matches(stage_id: StageId) -> Vec<Self> {
        let stage: Stage = Stage::fetch_from_db(&stage_id);
        let start_date: Date = stage.get_previous_start_date();
        let end_date: Date = stage.get_next_end_date();

        let mut games: Vec<Self> = Vec::new();
        for (date_s, date_games) in GAMES.lock().unwrap().iter() {
            let date: Date = db_string_to_date(date_s);

            // We can skip this date if it does not fit.
            if date < start_date || date > end_date { continue; }

            for game in date_games.values() {
                if game.stage_id == stage_id { games.push(game.clone()) }
            }
        }

        return games;
    }

    // Get matches of a stage arranged by date.
    pub fn fetch_stage_matches_by_date(stage_id: StageId) -> Vec<(Date, Vec<Game>)> {
        let stage: Stage = Stage::fetch_from_db(&stage_id);
        let start_date: Date = stage.get_previous_start_date();
        let end_date: Date = stage.get_next_end_date();

        let mut dates_and_games: Vec<(Date, Vec<Game>)> = Vec::new();
        for (date_s, date_games) in GAMES.lock().unwrap().iter() {
            let date: Date = db_string_to_date(date_s);

            // We can skip this date if it does not fit.
            if date < start_date || date > end_date { continue; }

            let mut stage_games: Vec<Game> = Vec::new();
            for game in date_games.values() {
                if game.stage_id == stage_id { stage_games.push(game.clone()) }
            }

            if date_games.len() > 0 {
                dates_and_games.push((date, stage_games));
            }
        }

        dates_and_games.sort_by(|a, b|a.0.cmp(&b.0));
        return dates_and_games;
    }

    pub fn fetch_from_db<S: AsRef<str>>(date: S, id: &GameId) -> Self {
        let games = GAMES.lock().unwrap();
        let date: &HashMap<u8, Game> = games.get(date.as_ref())
            .expect(&format!("no Game with date {:?}", date.as_ref()));

        date.get(id).expect(&format!("no Game with id {id}")).clone()
    }

    // Update the Game to database. Create ID if one does not already exist.
    pub fn save(&mut self) {
        let mut games: HashMap<String, HashMap<u8, Self>> = GAMES.lock().unwrap().clone();

        let date: &mut HashMap<u8, Self> = match games.get_mut(&self.date) {
            Some(d) => d,
            None => {
                games.insert(self.date.clone(), HashMap::new());
                games.get_mut(&self.date).unwrap()
            }
        };

        if self.id == 0 { self.create_id(date.len() + 1); }

        date.insert(self.id, self.clone());
        *GAMES.lock().unwrap() = games;
    }

    // Delete the Game from the database.
    pub fn delete_from_db(&self) {
        let mut games = GAMES.lock().unwrap();
        let date: &mut HashMap<u8, Game> = games.get_mut(&self.date).expect(&format!("Could not find date {:?} in GAMES", &self.date));

        date.remove(&self.id);
        *GAMES.lock().unwrap() = games.clone();
    }

    // Make sure the game does not contain illegal values.
    fn is_valid(&self) -> bool {
        self.home.is_valid() && self.away.is_valid() && self.get_rules().is_valid()
    }

    // Get the game rules.
    fn get_rules(&self) -> Rules {
        Stage::fetch_from_db(&self.stage_id).match_rules
    }

    // Get the stage of the game.
    fn get_stage(&self) -> Stage {
        Stage::fetch_from_db(&self.stage_id)
    }
}

// Functional.
impl Game {
    // Check if the team with this ID is playing in the match.
    pub fn is_team_playing(&self, team_id: TeamId) -> bool {
        team_id == self.home.team_id || team_id == self.away.team_id
    }

    // Call when both teams must submit their lineups.
    fn get_team_lineups(&mut self) {
        let mut home: Team = self.home.get_team();
        let mut away: Team = self.away.get_team();

        // Player lineup should not be auto-built.
        home.auto_build_lineup();
        away.auto_build_lineup();

        self.home.lineup = home.lineup;
        self.away.lineup = away.lineup;
    }

    // Do things like submitting lineups.
    fn do_pre_game_tasks(&mut self) {
        self.get_team_lineups();
    }

    // Do everything that needs to be done after the game is concluded.
    fn do_post_game_tasks(&mut self) {
        self.attacker = None;
        self.defender = None;
        self.save();

        // Update the teams' stage datas.
        let mut stage: Stage = self.get_stage();
        let had_overtime: bool = self.has_overtime();

        self.home.update_stagedata(&self.away, had_overtime, stage.teams.get_mut(&self.home.team_id).unwrap());
        self.away.update_stagedata(&self.home, had_overtime, stage.teams.get_mut(&self.away.team_id).unwrap());

        stage.save();
    }

    // Play the game.
    pub fn play(&mut self) {
        self.do_pre_game_tasks();
        self.simulate();    // The actual game is played here.
        self.do_post_game_tasks();
    }

    // Simulate a game of ice hockey.
    fn simulate(&mut self) {
        // Regular time.
        while !self.is_regular_time_over() {
            self.simulate_regular_period();
        }

        // Overtime.
        while !self.is_overtime_over() {
            self.simulate_overtime_period();
        }
    }

    // Simulate a period of ice hockey.
    fn simulate_regular_period(&mut self) {
        while !self.is_period_over() {
            self.simulate_second();
        }

        self.clock.next_period();
    }

    fn simulate_overtime_period(&mut self) {
        while !self.is_overtime_period_over() {
            self.simulate_second();
        }

        self.clock.next_period();
    }

    // Simulate a second of ice hockey.
    fn simulate_second(&mut self) {
        self.change_players_on_ice();
        self.change_puck_possession();
        self.attempt_shot();

        self.update_teamdata();
        self.clock.advance();
    }

    // Change the players on ice for home and away teams.
    fn change_players_on_ice(&mut self) {
        self.home.change_players_on_ice();
        self.away.change_players_on_ice();
    }

    // Change which team has the puck.
    fn change_puck_possession(&mut self) {
        let modifier: f64 = self.home.players_on_ice.as_ref().unwrap().get().get_skaters_ability_ratio(
            self.away.players_on_ice.as_ref().unwrap());

        if logic_event::Type::fetch_from_db(&logic_event::Id::PuckPossessionChange).get_outcome(modifier) {
            self.attacker = Some(self.home.clone());
            self.defender = Some(self.away.clone());
        }
        else {
            self.attacker = Some(self.away.clone());
            self.defender = Some(self.home.clone());
        }
    }

    // The attacking team attempts to shoot the puck.
    fn attempt_shot(&mut self) {
        let attacker: &mut TeamData = self.attacker.as_mut().unwrap();
        let attacker_players: &team::PlayersOnIce = attacker.players_on_ice.as_ref().unwrap();
        let defender_players = self.defender.as_ref().unwrap().players_on_ice.as_ref().unwrap();

        let modifier: f64 = attacker_players.get().get_skaters_ability_ratio(defender_players);
        let success: bool = logic_event::Type::fetch_from_db(&logic_event::Id::ShotAtGoal).get_outcome(modifier);

        if success {
            let mut shot: Shot = Shot::build(self.clock.clone(), attacker_players, defender_players);
            shot.create_shooter_and_assisters();
            shot.calculate_goal();
            attacker.shots.push(shot);
        }
    }

    // Update the attacker and defender teamdata.
    fn update_teamdata(&mut self) {
        let attacker: &TeamData = self.attacker.as_ref().unwrap();
        let defender: &TeamData = self.defender.as_ref().unwrap();

        if attacker.team_id == self.home.team_id {
            self.home = attacker.clone();
            self.away = defender.clone();
        }
        else {
            self.away = attacker.clone();
            self.home = defender.clone();
        }
    }

    // Get the name of the game if it has not begun.
    // Get the score as well if it has.
    pub fn get_name_and_score_if_started(&self) -> String {
        match self.clock == Clock::default() {
            true => self.get_name(),
            _ => self.get_name_and_score()
        }
    }

    // Get the home and away team names.
    fn get_name(&self) -> String {
        format!("{} - {}", self.home.get_team().name, self.away.get_team().name)
    }

    // Get the score of the game.
    fn get_score(&self) -> String {
        let ot = match self.has_overtime() {
            true => " OT",
            _ => ""
        };

        format!("{} - {}{}", self.home.get_goal_amount(), self.away.get_goal_amount(), ot)
    }

    // Get the home and away team names, as well as the game score.
    pub fn get_name_and_score(&self) -> String {
        format!("{} {} {}", self.home.get_team().name, self.get_score(), self.away.get_team().name)
    }
}

// Clock-related functions.
impl Game {
    // Get a minutes-seconds representation of the time that has passed in the game.
    fn game_time_to_string(&self) -> String {
        Clock::time_to_string(self.get_game_total_seconds())
    }

    // Get the total seconds that have passed in the game.
    fn get_game_total_seconds(&self) -> u32 {
        (self.clock.periods_completed as u32) * (self.get_rules().period_length as u32) + (self.clock.period_total_seconds as u32)
    }

    // Get the amount of minutes that have passed in the game.
    fn get_game_minutes(&self) -> u32 {
        self.get_game_total_seconds() / 60
    }

    // Get the amount of seconds that have passed in the game, after full minutes have been taken out.
    fn get_game_seconds(&self) -> u8 {
        convert::u32_to_u8(self.get_game_total_seconds() % 60)
    }

    // Check if the game is over.
    fn is_game_over(&self) -> bool {
        return self.is_regular_time_over() && self.is_overtime_over()
    }

    // Check if the regular time of the game is over.
    fn is_regular_time_over(&self) -> bool {
        self.clock.periods_completed >= self.get_rules().periods
    }

    // Check if the currently ongoing period is over.
    fn is_period_over(&self) -> bool {
        self.clock.period_total_seconds >= self.get_rules().period_length
    }

    // Check if the overtime period is over.
    fn is_overtime_period_over(&self) -> bool {
        return self.is_overtime_over() || self.is_period_over()
    }

    // Check if the overtime is over.
    fn is_overtime_over(&self) -> bool {
        // Always ends if teams are not tied.
        if self.home.get_goal_amount() != self.away.get_goal_amount() {
            return true;
        }

        if self.get_rules().continuous_overtime {
            return false;
        }

        return self.get_time_expired_in_overtime() >= (self.get_rules().overtime_length as i32);
    }

    // How much overtime has been played so far.
    // Negative values mean that the regular time is still ongoing.
    fn get_time_expired_in_overtime(&self) -> i32 {
        convert::u32_to_i32(self.get_game_total_seconds()) - (self.get_rules().get_regular_time() as i32)
    }

    // Check if the game has or had overtime.
    fn has_overtime(&self) -> bool {
        self.get_time_expired_in_overtime() > 0
    }
}

// Tests.
impl Game {
    // Generate an ascetic infodump about which team scored and when.
    pub fn get_simple_boxscore(&self) -> String {
        let rules: Rules = self.get_rules();

        struct BoxscoreGoal {
            team: String,
            score_info: String,
            total_seconds: u32,
        }

        let home_name: String = self.home.get_team().name;
        let away_name: String = self.away.get_team().name;

        let mut events: Vec<BoxscoreGoal> = Vec::new();
        for goal in self.home.shots.iter() {
            if goal.is_goal {
                events.push(BoxscoreGoal {
                    team: home_name.clone(),
                    score_info: goal.scorer_and_assists_to_string(),
                    total_seconds: (goal.event.time.periods_completed as u32) *
                        (rules.period_length as u32) + (goal.event.time.period_total_seconds as u32),
                });
            }
        }

        for goal in self.away.shots.iter() {
            if goal.is_goal {
                events.push(BoxscoreGoal {
                    team: away_name.clone(),
                    score_info: goal.scorer_and_assists_to_string(),
                    total_seconds: (goal.event.time.periods_completed as u32) *
                        (rules.period_length as u32) + (goal.event.time.period_total_seconds as u32),
                });
            }
        }

        events.sort_by(|a: &BoxscoreGoal, b: &BoxscoreGoal| a.total_seconds.cmp(&b.total_seconds));

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

            boxscore += &format!("{}: {} - {} {} - {}\n", Clock::time_to_string(event.total_seconds), home_goal_counter, away_goal_counter, event.team, event.score_info);
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

#[derive(Default, Clone, PartialEq)]
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
    fn time_to_string(seconds: u32) -> String {
        format!("{}:{:0>2}", seconds / 60, seconds % 60)
    }

    // Get a minutes-seconds representation of the time that has passed in the period.
    fn period_time_to_string(&self) -> String {
        Clock::time_to_string(self.period_total_seconds as u32)
    }

    // Get the amount of minutes that have passed in the period.
    fn get_period_minutes(&self) -> u8 {
        convert::u16_to_u8(self.period_total_seconds / 60)
    }

    // Get the amount of seconds that have passed in the period, after full minutes have been taken out.
    fn get_period_seconds(&self) -> u8 {
        convert::u16_to_u8(self.period_total_seconds % 60)
    }

    // Advance time by one second.
    fn advance(&mut self) {
        self.period_total_seconds += 1;
    }

    // Move on to the next period.
    fn next_period(&mut self) {
        self.reset_period_time();
        self.periods_completed += 1;
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