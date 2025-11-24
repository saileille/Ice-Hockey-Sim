pub mod event;
pub mod team;
mod cache;

use rand::rngs::ThreadRng;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{Decode, Encode, FromRow, Sqlite, encode::IsNull, error::BoxDynError, sqlite::{SqliteArgumentValue, SqliteTypeInfo, SqliteValueRef}};
use time::Date;

use crate::{competition::{Competition, format::Format, season::team::TeamSeason}, event as logic_event, match_event::{cache::{Attacker, GameCache}, event::Shot}, time::date_to_string, types::{Db, GameId, SeasonId, TeamId, convert}};

#[derive(Debug, Clone)]
#[derive(FromRow)]
pub struct Game {
    id: GameId,
    pub date: Date,
    home_id: TeamId,
    away_id: TeamId,
    #[sqlx(json)]
    clock: Clock,
    season_id: SeasonId,

    #[sqlx(skip)]
    cache: GameCache,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            id: GameId::default(),
            date: Date::MIN,
            home_id: TeamId::default(),
            away_id: TeamId::default(),
            clock: Clock::default(),
            season_id: SeasonId::default(),
            cache: GameCache::default(),
        }
    }
}

// Basics.
impl Game {
    // Get the next ID to use.
    async fn next_id(db: &Db) -> GameId {
        let max: Option<GameId> = sqlx::query_scalar("SELECT max(id) FROM Game").fetch_one(db).await.unwrap();
        match max {
            Some(n) => n + 1,
            _ => 1,
        }
    }

    async fn build(db: &Db, home_id: TeamId, away_id: TeamId, season_id: SeasonId, date: Date) -> Self {
        Self {
            id: Self::next_id(db).await,
            date,
            home_id,
            away_id,
            season_id,

            ..Default::default()
        }
    }

    pub async fn build_and_save(db: &Db, home_id: TeamId, away_id: TeamId, season_id: SeasonId, date: Date) -> Self {
        let game = Self::build(db, home_id, away_id, season_id, date).await;
        game.save(db).await;

        return game;
    }

    // Save.
    async fn save(&self, db: &Db) {
        sqlx::query(
            "INSERT INTO Game
            (id, date, home_id, away_id, clock, season_id)
            VALUES ($1, $2, $3, $4, $5, $6)"
        ).bind(self.id)
        .bind(self.date)
        .bind(self.home_id)
        .bind(self.away_id)
        .bind(self.clock)
        .bind(self.season_id)
        .execute(db).await.unwrap();
    }

    // Get the game rules.
    async fn rules(&self, db: &Db) -> Rules {
        let format: Format = sqlx::query_scalar(
            "SELECT format FROM Competition
            INNER JOIN Season ON Season.comp_id = Competition.id
            INNER JOIN Game ON Game.season_id = Season.id
            WHERE Game.id = $1"
        ).bind(self.id)
        .fetch_one(db).await.unwrap();

        return format.match_rules;
    }

    async fn competition(&self, db: &Db) -> Competition {
        sqlx::query_as(
            "SELECT Competition.* FROM Competition
            INNER JOIN Season ON Season.comp_id = Competition.id
            WHERE Season.id = $1"
        ).bind(self.season_id)
        .fetch_one(db).await.unwrap()
    }

    // Get nice data for a competition screen.
    pub async fn comp_screen_package(&self, db: &Db) -> serde_json::Value {
        json!({
            "home": self.cache.home.game_data.comp_screen_package(db).await,
            "away": self.cache.away.game_data.comp_screen_package(db).await,
            "date": date_to_string(self.date),
            "had_overtime": self.has_overtime(),
            "is_over": self.clock != Clock::default()
        })
    }
}

// Functional.
impl Game {
    // Call when both teams must submit their lineups.
    async fn submit_team_lineups(&mut self, db: &Db) {
        // The human's lineup should not be forced to autobuild, eventually.
        self.cache.build_lineups(db).await;
    }

    // Do things like submitting lineups.
    async fn do_pre_game_tasks(&mut self, db: &Db) {
        self.cache = GameCache::build(db, self.id, self.home_id, self.away_id, self.rules(db).await).await;
        self.submit_team_lineups(db).await;

        if !self.cache.home.game_data.lineup.is_full() {
            println!("Lineup of {} is not full.", self.cache.home.team.full_name);
            println!("{:#?}", self.cache.home.team.lineup);
            println!("{:#?}", self.cache.home.team.player_needs);
        }
        if !self.cache.away.game_data.lineup.is_full() {
            println!("Lineup of {} is not full.", self.cache.away.team.full_name);
            println!("{:#?}", self.cache.away.team.lineup);
            println!("{:#?}", self.cache.away.team.player_needs);
        }
    }

    // Do everything that needs to be done after the game is concluded.
    async fn do_post_game_tasks(&mut self, db: &Db) {
        let mut o_comp = Some(self.competition(db).await);
        let (home_data, away_data) = TeamSeason::season_data_from_game(&self.cache.home.game_data, &self.cache.away.game_data, self.has_overtime());

        // Updating the team data to all parent competitions of this competition.
        while o_comp.is_some() {
            let comp = o_comp.as_ref().unwrap();
            let season = comp.current_season(db).await;

            let mut home = season.team_with_id(db, self.home_id).await;
            let mut away = season.team_with_id(db, self.away_id).await;
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
        let mut rng = rand::rng();
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
        self.change_puck_possession(rng);
        Self::attempt_shot(rng, &mut self.cache, self.clock);

        self.clock.advance();
    }

    // Change the players on ice for home and away teams.
    fn change_players_on_ice(&mut self, rng: &mut ThreadRng) {
        self.cache.home.lineup.change_players_on_ice(rng);
        self.cache.away.lineup.change_players_on_ice(rng);
    }

    // Change which team has the puck.
    fn change_puck_possession(&mut self, rng: &mut ThreadRng) {
        let modifier = self.cache.home.lineup.players_on_ice.skaters_ability_ratio(
            &self.cache.away.lineup.players_on_ice
        );

        if logic_event::Type::fetch_from_db(&logic_event::Id::PuckPossessionChange).get_outcome(rng, modifier) {
            self.cache.attacker = Attacker::Home;
        }
        else {
            self.cache.attacker = Attacker::Away;
        }
    }

    // The attacking team attempts to shoot the puck.
    fn attempt_shot(rng: &mut ThreadRng, cache: &mut GameCache, clock: Clock) {
        let (attacker, defender) = match cache.attacker {
            Attacker::Home => (&cache.home, &cache.away),
            Attacker::Away => (&cache.away, &cache.home),
            _ => panic!("attacker cannot be null when attempting a shot")
        };

        let modifier = attacker.lineup.players_on_ice.skaters_ability_ratio(
            &defender.lineup.players_on_ice
        );

        let success = logic_event::Type::fetch_from_db(&logic_event::Id::ShotAtGoal).get_outcome(rng, modifier);

        if success {
            let shot = Shot::simulate(rng, clock, &attacker.lineup.players_on_ice, &defender.lineup.players_on_ice);

            if cache.home.team.id == attacker.team.id {
                cache.home.game_data.shots.push(shot);
            }
            else {
                cache.away.game_data.shots.push(shot);
            }
        }
    }
}

// Clock-related functions.
impl Game {
    // Get the total seconds that have passed in the game.
    fn total_seconds(&self) -> u32 {
        (self.clock.periods_completed as u32) * (self.cache.rules.period_length as u32) + (self.clock.period_total_seconds as u32)
    }

    // Check if the regular time of the game is over.
    fn is_regular_time_over(&self) -> bool {
        self.clock.periods_completed >= self.cache.rules.periods
    }

    // Check if the currently ongoing period is over.
    fn is_period_over(&self) -> bool {
        self.clock.period_total_seconds >= self.cache.rules.period_length
    }

    // Check if the overtime period is over.
    fn is_overtime_period_over(&self) -> bool {
        return self.is_overtime_over() || self.is_period_over()
    }

    // Check if the overtime is over.
    fn is_overtime_over(&self) -> bool {
        // Always ends if teams are not tied.
        if self.cache.home.game_data.goals() != self.cache.away.game_data.goals() {
            return true;
        }

        if self.cache.rules.continuous_overtime {
            return false;
        }

        return self.time_expired_in_overtime() >= (self.cache.rules.overtime_length as i32);
    }

    // How much overtime has been played so far.
    // Negative values mean that the regular time is still ongoing.
    fn time_expired_in_overtime(&self) -> i32 {
        convert::int::<u32, i32>(self.total_seconds()) - (self.cache.rules.regular_time() as i32)
    }

    // Check if the game has or had overtime.
    pub fn has_overtime(&self) -> bool {
        self.time_expired_in_overtime() > 0
    }
}

#[derive(Debug, Serialize, Deserialize)]
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
        Self {
            periods: periods,
            period_length: period_length,
            overtime_length: overtime_length,
            continuous_overtime: continous_overtime,
        }
    }
}

// Functional.
impl Rules {
    // Get the total regular time of the game in seconds.
    fn regular_time(&self) -> u16 {
        (self.periods as u16) * self.period_length
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[derive(Default, Clone, Copy, PartialEq)]
pub struct Clock {
    periods_completed: u8,
    period_total_seconds: u16,
}

impl sqlx::Type<Sqlite> for Clock {
    fn type_info() -> SqliteTypeInfo {
        <String as sqlx::Type<Sqlite>>::type_info()
    }
}

impl<'q> Encode<'q, Sqlite> for Clock {
    fn encode(self, buf: &mut Vec<SqliteArgumentValue<'q>>) -> Result<IsNull, BoxDynError> {
        Encode::<Sqlite>::encode(serde_json::to_string(&self).unwrap(), buf)
    }

    fn encode_by_ref(&self, buf: &mut Vec<SqliteArgumentValue<'q>>) -> Result<IsNull, BoxDynError> {
        Encode::<Sqlite>::encode(serde_json::to_string(self).unwrap(), buf)
    }
}

impl<'r> Decode<'r, Sqlite> for Clock {
    fn decode(value: SqliteValueRef<'r>) -> Result<Self, BoxDynError> {
        let json = <serde_json::Value as Decode<Sqlite>>::decode(value)?;
        Ok(serde_json::from_value(json)?)
    }
}

// Functional.
impl Clock {
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
}