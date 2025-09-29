use rand::Rng;

use crate::person;
use crate::team;

pub struct Game<'a> {
    home: TeamData<'a>,
    away: TeamData<'a>,
    gameclock: GameClock,
    rules: Rules,
}

pub fn build_game() -> Game<'static> {
    Game {
        home: build_team_data(String::from("Home")),
        away: build_team_data(String::from("Away")),
        gameclock: build_gameclock(0, 0),
        rules: build_rules(3, 1200),
    }
}

impl Game<'_> {
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
        while (self.gameclock.seconds as u16) < self.rules.period_length {
            let has_puck;
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
                has_puck.shots.push(build_shot(self.gameclock.periods_completed, self.gameclock.seconds, true));
            }

            self.gameclock.advance();
        }
    }

    fn get_name(&self) -> String {
        // Get the home and away team names.
        format!("{} - {}", self.home.team.name, self.away.team.name)
    }

    pub fn get_name_and_score(&self) -> String {
        // Get the home and away team names, as well as the game score.
        format!("{} {} - {} {}", self.home.team.name, self.home.get_goal_amount(), self.away.get_goal_amount(), self.away.team.name)
    }

    pub fn get_simple_boxscore(&self) -> String {
        // Generate an ascetic infodump about which team scored and when.
        struct BoxscoreGoal<'a> {
            goal: &'a Shot<'a>,
            team: &'a team::Team<'a>,
        }

        let mut events = Vec::new();
        for goal in self.home.shots.iter() {
            events.push(BoxscoreGoal {
                goal: goal,
                team: &self.home.team,
            });
        }

        for goal in self.away.shots.iter() {
            events.push(BoxscoreGoal {
                goal: goal,
                team: &self.away.team,
            });
        }

        events.sort_by(|a, b| a.goal.event.time.get_total_time(self.rules.period_length).cmp(&b.goal.event.time.get_total_time(self.rules.period_length)));

        let mut boxscore = String::new();
        let mut home_goal_counter: u8 = 0;
        let mut away_goal_counter: u8 = 0;
        for event in events.iter() {
            if event.team.name == self.home.team.name {
                home_goal_counter += 1;
            }
            else {
                away_goal_counter += 1;
            }

            boxscore += &format!("{}: {} {} - {}\n", event.goal.event.time.full_time_to_string(self.rules.period_length), event.team.name, home_goal_counter, away_goal_counter);
        }

        return boxscore;
    }

    fn reset(&mut self) {
        // Reset the match by setting the time and team data back to default values.
        self.gameclock.reset();
        self.home.reset();
        self.away.reset();
    }

    // Test stuffs.

    fn generate_players(&mut self) {
        // Generate players for both teams.
        self.home.team.generate_roster(0, 0);
        self.away.team.generate_roster(0, 0);
    }
}

struct TeamData<'a> {
    team: team::Team<'a>,
    shots: Vec<Shot<'a>>,
    lineup: Vec<&'a person::Player<'a>>,
    players_on_ice: Vec<&'a person::Player<'a>>,
    penalties: Vec<String>, // Placeholder.
}

fn build_team_data(name: String) -> TeamData<'static> {
    TeamData {
        team: team::build_team(name),
        shots: Vec::new(),
        lineup: Vec::new(),
        players_on_ice: Vec::new(),
        penalties: Vec::new(),
    }
}

impl TeamData<'_> {
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

    fn build_lineup(&self) {
        // Build a lineup for the team.
    }
}

struct Event<'a> {
    time: GameClock,
    attacking_players: Vec<&'a person::Player<'a>>,
    defending_players: Vec<&'a person::Player<'a>>,
}

fn build_event(periods_completed: u8, seconds: u16) -> Event<'static> {
    Event {
        time: build_gameclock(periods_completed, seconds),
        attacking_players: Vec::new(),
        defending_players: Vec::new(),
    }
}

struct Shot<'a> {
    event: Event<'a>,
    is_goal: bool,
}

fn build_shot(periods_completed: u8, seconds: u16, is_goal: bool) -> Shot<'static> {
    Shot {
        event: build_event(periods_completed, seconds),
        is_goal: is_goal,
    }
}

struct Rules {
    periods: u8,
    period_length: u16
}

fn build_rules(periods: u8, period_length: u16) -> Rules {
    Rules {
        periods: periods,
        period_length: period_length,
    }
}

struct GameClock {
    periods_completed: u8,
    seconds: u16,
}

fn build_gameclock(periods_completed: u8, seconds: u16) -> GameClock {
    GameClock {
        periods_completed: periods_completed,
        seconds: seconds,
    }
}

impl GameClock {
    fn time_to_string_u16(seconds: u16) -> String {
        format!("{}:{:0>2}", seconds / 60, seconds % 60)
    }

    fn time_to_string_u32(seconds: u32) -> String {
        format!("{}:{:0>2}", seconds / 60, seconds % 60)
    }

    fn full_time_to_string(&self, period_length: u16) -> String {
        // Get a minutes-seconds representation.
        GameClock::time_to_string_u32(self.get_total_time(period_length))
    }

    fn period_time_to_string(&self) -> String {
        // Get a minutes-seconds representation.
        return GameClock::time_to_string_u16(self.seconds);
    }

    fn advance(&mut self) {
        // Advance time by one second.
        self.seconds += 1;
    }

    fn reset_period_time(&mut self) {
        // Reset the period clock.
        self.seconds = 0;
    }

    fn reset(&mut self) {
        // Reset the clock completely.
        self.seconds = 0;
        self.periods_completed = 0;
    }

    fn get_total_time(&self, period_length: u16) -> u32 {
        (self.periods_completed as u32) * (period_length as u32) + (self.seconds as u32)
    }
}