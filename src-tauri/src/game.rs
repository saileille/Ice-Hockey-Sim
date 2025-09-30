use rand::Rng;

use crate::database;
use crate::team::Team;

#[derive(Default)]
pub struct Game {
    home: TeamData,
    away: TeamData,
    gameclock: GameClock,
    rules: Rules,
}

impl Game { // Basics.
    pub fn new(home: usize, away: usize) -> Self {
        Game {
            home: TeamData::new(home),
            away: TeamData::new(away),
            gameclock: GameClock::new(0, 0),
            rules: Rules::new(3, 1200),
        }
    }
}

impl Game {
    // The chance a team has in getting a shot in at the goal.
    const BASE_SHOT_CHANCE: f64 = 60.0 / 3600.0;

    pub fn simulate(&mut self) {
        // Simulate a game of ice hockey.
        while self.gameclock.periods_completed < self.rules.periods {
            self.simulate_period();
            self.gameclock.reset_period_time();
            self.gameclock.periods_completed += 1;
        }
    }

    fn simulate_period(&mut self) {
        // Simulate a period of ice hockey.
        let mut rng = rand::rng();
        while (self.gameclock.period_total_seconds as u16) < self.rules.period_length {
            let has_puck: &mut TeamData;
            // let has_no_puck;
            
            if rng.random_bool(0.5) {
                has_puck = &mut self.home;
                // has_no_puck = &mut self.away;
            }
            else {
                has_puck = &mut self.away;
                // has_no_puck = &mut self.home;
            }
            
            if rng.random_bool(5.5 / 3600.0) {
                has_puck.shots.push(build_shot(self.gameclock.periods_completed, self.gameclock.period_total_seconds, true));
            }

            self.gameclock.advance();
        }
    }

    fn get_name(&self) -> String {
        // Get the home and away team names.
        format!("{} - {}", self.home.get_team_clone().name, self.away.get_team_clone().name)
    }

    pub fn get_name_and_score(&self) -> String {
        // Get the home and away team names, as well as the game score.
        format!("{} {} - {} {}", self.home.get_team_clone().name, self.home.get_goal_amount(), self.away.get_goal_amount(), self.away.get_team_clone().name)
    }

    fn reset(&mut self) {
        // Reset the match by setting the time and team data back to default values.
        self.gameclock.reset();
        self.home.reset();
        self.away.reset();
    }
}

impl Game { // Methods for testing phase.
    /*fn generate_players(&mut self) {
        // Generate players for both teams.
        self.home.team.as_mut().unwrap().generate_roster(0, 0);
        self.away.team.as_mut().unwrap().generate_roster(0, 0);
    }*/

    pub fn get_simple_boxscore(&self) -> String {
        // Generate an ascetic infodump about which team scored and when.
        struct BoxscoreGoal {
            goal: Shot,
            team: String,
        }

        let home_name: String = self.home.get_team_clone().name;
        let away_name: String = self.away.get_team_clone().name;

        let mut events: Vec<BoxscoreGoal> = Vec::new();
        for goal in self.home.shots.iter() {
            events.push(BoxscoreGoal {
                goal: goal.clone(),
                team: home_name.clone(),
            });
        }

        for goal in self.away.shots.iter() {
            events.push(BoxscoreGoal {
                goal: goal.clone(),
                team: away_name.clone(),
            });
        }

        events.sort_by(
            |a: &BoxscoreGoal, b: &BoxscoreGoal| 
            a.goal.event.time.get_game_total_seconds(self.rules.period_length)
            .cmp(&b.goal.event.time.get_game_total_seconds(self.rules.period_length))
        );

        let mut boxscore: String = String::new();
        let mut home_goal_counter: u8 = 0;
        let mut away_goal_counter: u8 = 0;
        for event in events.iter() {
            if event.team == home_name {
                home_goal_counter += 1;
            }
            else {
                away_goal_counter += 1;
            }

            boxscore += &format!("{}: {} {} - {}\n", event.goal.event.time.game_time_to_string(self.rules.period_length), event.team, home_goal_counter, away_goal_counter);
        }

        return boxscore;
    }
}

#[derive(Default)]
struct TeamData {
    team_id: usize,
    shots: Vec<Shot>,
    //lineup: team::LineUp<'a>,
    players_on_ice: Vec<usize>,
    penalties: Vec<String>, // Placeholder.
}

impl TeamData { // Basics.
    fn new(team_id: usize) -> Self {
        let mut team_data: TeamData = TeamData::default();
        team_data.team_id = team_id;
        return team_data;
    }
    
    fn get_team_clone(&self) -> Team {
        // Get a clone of the team.
        return database::TEAMS.lock().unwrap().get(&self.team_id).unwrap().clone();
    }
}

impl TeamData {
    fn get_shot_amount(&self) -> u8 {
        self.shots.len() as u8
    }

    fn get_goal_amount(&self) -> u8 {
        let mut goal_counter: u8 = 0;
        for shot in self.shots.iter() {
            if shot.is_goal {
                goal_counter += 1;
            }
        }

        return goal_counter;
    }

    fn reset(&mut self) {
        // Reset the TeamData.
        self.shots = Vec::new();
        self.players_on_ice = Vec::new();
        self.penalties = Vec::new();
    }
}

#[derive(Default, Clone)]
struct Event {
    time: GameClock,
    attacking_players: Vec<usize>,
    defending_players: Vec<usize>,
}

impl Event {
    fn new(periods_completed: u8, seconds: u16) -> Self {
        let mut event = Event::default();
        event.time = GameClock::new(periods_completed, seconds);
        return event;
    }
}

#[derive(Default, Clone)]
struct Shot {
    event: Event,
    is_goal: bool,
}

fn build_shot(periods_completed: u8, seconds: u16, is_goal: bool) -> Shot {
    Shot {
        event: Event::new(periods_completed, seconds),
        is_goal: is_goal,
    }
}

#[derive(Default)]
struct Rules {
    periods: u8,
    period_length: u16
}

impl Rules {
    fn new(periods: u8, period_length: u16) -> Self {
        Rules {
            periods: periods,
            period_length: period_length,
        }
    }
}

#[derive(Default, Clone)]
struct GameClock {
    periods_completed: u8,
    period_total_seconds: u16,
}

impl GameClock {    // Basics.
    fn new(periods_completed: u8, seconds: u16) -> Self {
        GameClock {
            periods_completed: periods_completed,
            period_total_seconds: seconds,
        }
    }
}

impl GameClock {
    fn time_to_string_u16(seconds: u16) -> String {
        format!("{}:{:0>2}", seconds / 60, seconds % 60)
    }

    fn time_to_string_u32(seconds: u32) -> String {
        format!("{}:{:0>2}", seconds / 60, seconds % 60)
    }

    fn game_time_to_string(&self, period_length: u16) -> String {
        // Get a minutes-seconds representation of the time that has passed in the game.
        GameClock::time_to_string_u32(self.get_game_total_seconds(period_length))
    }

    fn period_time_to_string(&self) -> String {
        // Get a minutes-seconds representation of the time that has passed in the period.
        GameClock::time_to_string_u16(self.period_total_seconds)
    }

    fn get_game_total_seconds(&self, period_length: u16) -> u32 {
        // Get the total seconds that have passed in the game.
        (self.periods_completed as u32) * (period_length as u32) + (self.period_total_seconds as u32)
    }
    
    fn get_period_minutes(&self) -> u8 {
        // Get the amount of minutes that have passed in the period.
        (self.period_total_seconds / 60) as u8
    }

    fn get_period_seconds(&self) -> u8 {
        // Get the amount of seconds that have passed in the period, after full minutes have been taken out.
        (self.period_total_seconds % 60) as u8
    }

    fn get_game_minutes(&self, period_length: u16) -> u16 {
        // Get the amount of minutes that have passed in the game.
        (self.get_game_total_seconds(period_length) / 60) as u16
    }

    fn get_game_seconds(&self, period_length: u16) -> u8 {
        // Get the amount of seconds that have passed in the game, after full minutes have been taken out.
        (self.get_game_total_seconds(period_length) % 60) as u8
    }

    fn advance(&mut self) {
        // Advance time by one second.
        self.period_total_seconds += 1;
    }

    fn reset_period_time(&mut self) {
        // Reset the period clock.
        self.period_total_seconds = 0;
    }

    fn reset(&mut self) {
        // Reset the clock completely.
        self.period_total_seconds = 0;
        self.periods_completed = 0;
    }
}