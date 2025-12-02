pub mod event;
pub mod team;

use rand::rngs::ThreadRng;
use sqlx::FromRow;
use time::Date;

use crate::logic::{self, competition::season::team::TeamSeason, event::Id, game::{event::Shot, team::TeamGame}, types::{CompetitionId, Db, GameId, GameSeconds, SeasonId, convert}};

enum Attacker {
    Home,
    Away,
}

#[derive(Debug, Clone)]
pub struct Game {
    pub id: GameId,
    pub date: Date,
    pub clock: Clock,
    pub season_id: SeasonId,

    // Non-database values.
    pub rules: Rules,
    pub home: TeamGame,
    pub away: TeamGame,
}

impl Game {
    pub async fn build_lineups(&mut self, db: &Db) {
        self.home.auto_build_lineup(db).await;
        self.away.auto_build_lineup(db).await;

        self.home.build_lineup_cache(db).await;
        self.away.build_lineup_cache(db).await;
    }

    // Call when both teams must submit their lineups.
    async fn submit_team_lineups(&mut self, db: &Db) {
        // The human's lineup should not be forced to autobuild, eventually.
        self.build_lineups(db).await;
    }

    // Do things like submitting lineups.
    async fn do_pre_game_tasks(&mut self, db: &Db) {
        self.submit_team_lineups(db).await;

        if !self.home.lineup.is_full() {
            println!("Lineup of {} is not full.", self.home.team_name(db).await);
            println!("{:#?}", self.home.lineup);
        }
        if !self.away.lineup.is_full() {
            println!("Lineup of {} is not full.", self.away.team_name(db).await);
            println!("{:#?}", self.away.lineup);
        }
    }

    // Do everything that needs to be done after the game is concluded.
    async fn do_post_game_tasks(&mut self, db: &Db) {
        // Save the played game to the database.
        self.overwrite(db).await;

        let mut o_comp = Some(self.competition(db).await);
        let (home_data, away_data) = TeamSeason::season_data_from_game(&self.home, &self.away, self.has_overtime());

        // Updating the team data to all parent competitions of this competition.
        while o_comp.is_some() {
            let comp = o_comp.as_ref().unwrap();
            let season = comp.current_season(db).await;

            let mut home = season.team_with_id(db, self.home.team_id).await;
            let mut away = season.team_with_id(db, self.away.team_id).await;
            home.update_and_save(db, &home_data).await;
            away.update_and_save(db, &away_data).await;

            // Updating the knockout round pairs.
            if season.knockout_round.is_some() {
                season.knockout_round.unwrap().update_teamdata(db, &home_data, &away_data, season.id).await;
            }

            o_comp = comp.parent(db).await;
        }
    }

    // Play the game.
    pub async fn play(&mut self, db: &Db) {
        self.do_pre_game_tasks(db).await;
        self.simulate();    // The actual game is played here.
        self.do_post_game_tasks(db).await;
    }

    // Simulate a game of ice hockey.
    fn simulate(&mut self) {
        // Regular time.
        let mut rng: ThreadRng = rand::rng();
        while !self.is_regular_time_over() {
            self.simulate_regular_period(&mut rng);
        }

        // Overtime.
        while !self.is_overtime_over() {
            self.simulate_overtime_period(&mut rng);
        }
    }

    // Simulate a period of ice hockey.
    fn simulate_regular_period(&mut self, rng: &mut ThreadRng) {
        while !self.is_period_over() {
            self.simulate_second(rng);
        }

        self.clock.next_period();
    }

    fn simulate_overtime_period(&mut self, rng: &mut ThreadRng) {
        while !self.is_overtime_period_over() {
            self.simulate_second(rng);
        }

        self.clock.next_period();
    }

    // Simulate a second of ice hockey.
    fn simulate_second(&mut self, rng: &mut ThreadRng) {
        self.change_players_on_ice(rng);
        let attacker = self.change_puck_possession(rng);
        // Self::attempt_shot(rng, self.id, self.clock, &mut self.cache);
        self.attempt_shot(rng, attacker);

        self.clock.advance();
    }

    // Change the players on ice for home and away teams.
    fn change_players_on_ice(&mut self, rng: &mut ThreadRng) {
        self.home.lineup_cache.change_players_on_ice(rng);
        self.away.lineup_cache.change_players_on_ice(rng);
    }

    // Change which team has the puck.
    fn change_puck_possession(&mut self, rng: &mut ThreadRng) -> Attacker {
        let modifier = self.home.lineup_cache.players_on_ice.skaters_ability_ratio(
            &self.away.lineup_cache.players_on_ice
        );

        if logic::event::Type::fetch_from_db(Id::PuckPossessionChange).get_outcome(rng, modifier) {
            return Attacker::Home;
        }
        else {
            return Attacker::Away;
        }
    }

    // The attacking team attempts to shoot the puck.
    // fn attempt_shot(rng: &mut ThreadRng, game_id: GameId, time: GameSeconds, cache: &mut GameCache) {
    fn attempt_shot(&mut self, rng: &mut ThreadRng, attacker: Attacker) {
        let (attacker, defender) = match attacker {
            Attacker::Home => (&self.home, &self.away),
            Attacker::Away => (&self.away, &self.home),
        };

        let modifier = attacker.lineup_cache.players_on_ice.skaters_ability_ratio(
            &defender.lineup_cache.players_on_ice
        );

        let success = logic::event::Type::fetch_from_db(Id::ShotAtGoal).get_outcome(rng, modifier);

        if success {
            // let shot = Shot::simulate(rng, game_id, attacker.team.id, defender.team.id, time, &attacker.lineup.players_on_ice, &defender.lineup.players_on_ice);
            let shot = Shot::simulate(rng, self, attacker, defender);

            // The shot must be saved to the database later.
            if self.home.team_id == attacker.team_id {
                self.home.shots.push(shot);
            }
            else {
                self.away.shots.push(shot);
            }
        }
    }

    // Get the total seconds that have passed in the game.
    pub fn total_seconds(&self) -> GameSeconds {
        (self.clock.periods_completed as GameSeconds) * self.rules.period_length + self.clock.period_seconds
    }

    // Check if the regular time of the game is over.
    pub fn is_regular_time_over(&self) -> bool {
        self.clock.periods_completed >= self.rules.periods
    }

    // Check if the currently ongoing period is over.
    pub fn is_period_over(&self) -> bool {
        self.clock.period_seconds >= self.rules.period_length
    }

    // Check if the overtime period is over.
    pub fn is_overtime_period_over(&self) -> bool {
        return self.is_overtime_over() || self.is_period_over()
    }

    // Check if the overtime is over.
    pub fn is_overtime_over(&self) -> bool {
        // Always ends if teams are not tied.
        if self.home.goals() != self.away.goals() {
            return true;
        }

        if self.rules.continuous_overtime {
            return false;
        }

        return self.time_expired_in_overtime() >= (self.rules.overtime_length as i16);
    }

    // How much overtime has been played so far.
    // Negative values mean that the regular time is still ongoing.
    fn time_expired_in_overtime(&self) -> i16 {
        convert::int::<GameSeconds, i16>(self.total_seconds()) -
        convert::int::<u16, i16>(self.rules.regular_time())
    }

    // Check if the game has or had overtime.
    pub fn has_overtime(&self) -> bool {
        self.time_expired_in_overtime() > 0
    }
}

#[derive(Debug)]
#[derive(Default, Clone)]
// A more advanced and intelligent game clock than a simple integer.
pub struct Clock {
    periods_completed: u8,
    period_seconds: u16,
}

impl Clock {
    // Advance time by one second.
    pub fn advance(&mut self) {
        self.period_seconds += 1;
    }

    // Move on to the next period.
    pub fn next_period(&mut self) {
        self.reset_period_time();
        self.periods_completed += 1;
    }

    // Reset the period clock.
    fn reset_period_time(&mut self) {
        self.period_seconds = 0;
    }
}

#[derive(Debug, Default, Clone)]
#[derive(FromRow)]
pub struct Rules {
    comp_id: CompetitionId,
    pub periods: u8,
    pub period_length: u16,
    pub overtime_length: u16,
    pub continuous_overtime: bool,
}

impl Rules {
    pub fn build(periods: u8, period_length: u16, overtime_length: u16, continous_overtime: bool) -> Self {
        Self {
            periods: periods,
            period_length: period_length,
            overtime_length: overtime_length,
            continuous_overtime: continous_overtime,

            ..Default::default()
        }
    }

    // Get the total regular time of the game in seconds.
    fn regular_time(&self) -> u16 {
        (self.periods as u16) * self.period_length
    }

    pub fn clock_from_seconds(&self, time: GameSeconds) -> Clock {
        Clock {
            periods_completed: convert::int::<GameSeconds, u8>(time / self.period_length),
            period_seconds: time % self.period_length,
        }
    }
}