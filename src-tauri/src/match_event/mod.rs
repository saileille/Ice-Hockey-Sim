pub mod event;
pub mod team;
mod cache;

use serde_json::json;

use crate::{
    competition::{season::team::TeamCompData, Competition}, database::COMPETITIONS, event as logic_event, match_event::cache::GameCache, types::{
        convert, CompetitionId, TeamId
    }
};
use self::{
    team::TeamGameData,
    event::Shot
};

#[derive(Debug)]
#[derive(Default, Clone)]
enum Attacker {
    #[default]
    Null,
    Home,
    Away,
}

#[derive(Debug)]
#[derive(Default, Clone)]
pub struct Game {
    pub date: String,
    pub home: TeamGameData,
    pub away: TeamGameData,
    clock: Clock,
    comp_id: CompetitionId,
    cache: Option<GameCache>,
    attacker: Attacker,
}

// Basics.
impl Game {
    pub fn build(home: &TeamCompData, away: &TeamCompData, comp_id: CompetitionId, date: &str) -> Self {
        let mut game = Game::default();
        game.home = TeamGameData::build(home);
        game.away = TeamGameData::build(away);
        game.clock = Clock::default();
        game.comp_id = comp_id;
        game.date = date.to_string();

        return game;
    }

    // Make sure the game does not contain illegal values.
    fn is_valid(&self) -> bool {
        self.home.is_valid() && self.away.is_valid() && self.get_rules().is_valid()
    }

    // Get the game rules.
    fn get_rules(&self) -> Rules {
        COMPETITIONS.lock().unwrap().get(&self.comp_id).unwrap().format.as_ref().unwrap().match_rules.clone()
    }

    // Get the competition of the game.
    fn get_comp(&self) -> Competition {
        Competition::fetch_from_db(&self.comp_id)
    }

    // Get nice data for a competition screen.
    pub fn get_comp_screen_json(&self) -> serde_json::Value {
        json!({
            "home": self.home.get_comp_screen_json(),
            "away": self.away.get_comp_screen_json(),
            "date": self.date,
            "had_overtime": self.has_overtime(),
            "is_over": self.clock != Clock::default()
        })
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
        let cache = self.cache.as_mut().unwrap();

        // The human's lineup should not be forced to autobuild, eventually.
        cache.home.team.auto_build_lineup();
        cache.away.team.auto_build_lineup();

        self.home.lineup = cache.home.team.lineup.clone();
        self.away.lineup = cache.away.team.lineup.clone();

        self.cache.as_mut().unwrap().build_lineups(&self.home.lineup, &self.away.lineup);
    }

    // Do things like submitting lineups.
    fn do_pre_game_tasks(&mut self) {
        self.cache = Some(GameCache::build(&self.home, &self.away, &self.get_rules()));
        self.get_team_lineups();

        if !self.home.lineup.is_full() {
            let team = self.home.get_team();
            println!("Lineup of {} is not full.", team.name);
            println!("{:#?}", team.lineup);
            println!("{:#?}", team.player_needs);
        }
        if !self.away.lineup.is_full() {
            let team = self.away.get_team();
            println!("Lineup of {} is not full.", team.name);
            println!("{:#?}", team.lineup);
            println!("{:#?}", team.player_needs);
        }
    }

    // Do everything that needs to be done after the game is concluded.
    fn do_post_game_tasks(&mut self) {
        self.cache = None;

        // Update the teams' comp datas.
        // self.get_comp().update_teamdata(&self.home, &self.away, self.has_overtime());
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
        Self::attempt_shot(&mut self.home, &mut self.away, &self.clock, self.cache.as_ref().unwrap(), &self.attacker);

        self.clock.advance();
    }

    // Change the players on ice for home and away teams.
    fn change_players_on_ice(&mut self) {
        self.cache.as_mut().unwrap().home.lineup.change_players_on_ice();
        self.cache.as_mut().unwrap().away.lineup.change_players_on_ice();
    }

    // Change which team has the puck.
    fn change_puck_possession(&mut self) {
        let modifier = self.cache.as_ref().unwrap().home.lineup.players_on_ice.get_skaters_ability_ratio(
            &self.cache.as_ref().unwrap().away.lineup.players_on_ice
        );

        if logic_event::Type::fetch_from_db(&logic_event::Id::PuckPossessionChange).get_outcome(modifier) {
            self.attacker = Attacker::Home;
        }
        else {
            self.attacker = Attacker::Away;
        }
    }

    // The attacking team attempts to shoot the puck.
    // fn attempt_shot(&mut self) {
    fn attempt_shot(home: &mut TeamGameData, away: &mut TeamGameData, clock: &Clock, cache: &GameCache, attacker: &Attacker) {
        let (attacker, defender) = match attacker {
            Attacker::Home => (&cache.home, &cache.away),
            Attacker::Away => (&cache.away, &cache.home),
            _ => panic!("attacker cannot be null when attempting a shot")
        };

        let modifier = attacker.lineup.players_on_ice.get_skaters_ability_ratio(
            &defender.lineup.players_on_ice
        );

        let success = logic_event::Type::fetch_from_db(&logic_event::Id::ShotAtGoal).get_outcome(modifier);

        if success {
            let shot = Shot::simulate(clock.clone(), &attacker.lineup.players_on_ice, &defender.lineup.players_on_ice);

            if home.team_id == attacker.team.id {
                home.shots.push(shot);
            }
            else {
                away.shots.push(shot);
            }
        }
    }

    // Get the home and away team names.
    pub fn get_name(&self) -> String {
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
        let rules;
        let rules_ref;

        if self.cache.is_none() {
            rules = self.get_rules();
            rules_ref = &rules;
        }
        else {
            rules_ref = &self.cache.as_ref().unwrap().rules;
        }

        (self.clock.periods_completed as u32) * (rules_ref.period_length as u32) + (self.clock.period_total_seconds as u32)
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
        self.clock.periods_completed >= self.cache.as_ref().unwrap().rules.periods
    }

    // Check if the currently ongoing period is over.
    fn is_period_over(&self) -> bool {
        self.clock.period_total_seconds >= self.cache.as_ref().unwrap().rules.period_length
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

        if self.cache.as_ref().unwrap().rules.continuous_overtime {
            return false;
        }

        return self.get_time_expired_in_overtime() >= (self.cache.as_ref().unwrap().rules.overtime_length as i32);
    }

    // How much overtime has been played so far.
    // Negative values mean that the regular time is still ongoing.
    fn get_time_expired_in_overtime(&self) -> i32 {
        let rules;
        let rules_ref;

        if self.cache.is_none() {
            rules = self.get_rules();
            rules_ref = &rules;
        }
        else {
            rules_ref = &self.cache.as_ref().unwrap().rules;
        }

        convert::u32_to_i32(self.get_game_total_seconds()) - (rules_ref.get_regular_time() as i32)
    }

    // Check if the game has or had overtime.
    pub fn has_overtime(&self) -> bool {
        self.get_time_expired_in_overtime() > 0
    }
}

// Tests.
impl Game {
    // Generate an ascetic infodump about which team scored and when.
    pub fn get_simple_boxscore(&self) -> String {
        let rules = self.get_rules();

        struct BoxscoreGoal {
            team: String,
            score_info: String,
            total_seconds: u32,
        }

        let home_name = self.home.get_team().name;
        let away_name = self.away.get_team().name;

        let mut events = Vec::new();
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

        let mut boxscore = String::new();
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

#[derive(Debug, serde::Serialize)]
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

#[derive(Debug, serde::Serialize)]
#[derive(Default, Clone, PartialEq)]
pub struct Clock {
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