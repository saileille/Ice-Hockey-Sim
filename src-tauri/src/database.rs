// The game database.
use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex}
};
use time::{macros::date, Date};
use lazy_static::lazy_static;

use crate::{
    types,
    event,
    team::Team,
    person::player::{
        Player,
        position::{Position, PositionId}
    },
    country::Country,
    competition::{
        Competition,
        stage::{Stage, round_robin::RoundRobin}
    },
    match_event,
    io
};

// The current date in the game.
pub static TODAY: LazyLock<Mutex<Date>> = LazyLock::new(|| Mutex::new(date!(2025-08-01)));

pub static COUNTRIES: LazyLock<Mutex<HashMap<types::CountryId, Country>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
pub static COMPETITIONS: LazyLock<Mutex<HashMap<types::CompetitionId, Competition>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
pub static STAGES: LazyLock<Mutex<HashMap<types::StageId, Stage>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

// NOTE: GAMES is exceptional in that the games are stored first based on date and then GameId.
pub static GAMES: LazyLock<Mutex<
    HashMap<String, HashMap<types::GameId, match_event::Game>>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

pub static TEAMS: LazyLock<Mutex<HashMap<types::TeamId, Team>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
pub static PLAYERS: LazyLock<Mutex<HashMap<types::PlayerId, Player>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

lazy_static! {
    pub static ref POSITIONS: HashMap<PositionId, Position> = {
         let p = HashMap::from([
            (PositionId::default(), Position::default()),
            (PositionId::Goalkeeper, Position::build(
                PositionId::Goalkeeper, 0
            )),
            (PositionId::Defender, Position::build(
                PositionId::Defender, 0
            )),
            (PositionId::LeftWinger, Position::build(
                PositionId::LeftWinger, 0
            )),
            (PositionId::Centre, Position::build(
                PositionId::Centre, 0
            )),
            (PositionId::RightWinger, Position::build(
                PositionId::RightWinger, 0
            )),
        ]);
        return p;
    };

    pub static ref EVENT_TYPES: HashMap<event::Id, event::Type> = {
        let e = HashMap::from([
            // Chance of home team getting the puck. Failure means it goes to away team.
            (event::Id::PuckPossessionChange, event::Type::build(0.1, 0.5, 0.9)),

            // Chance of attacking team to get a shot at the goal.
            // Minimum chance is 10 times as low as the equilibrium, maximum chance is 10 times as high.
            (event::Id::ShotAtGoal, event::Type::build(5.6 / 3600.0, 56.0 / 3600.0, 560.0 / 3600.0)),

            // Chance of a shot going in goal.
            // NOTE: min_boundary and max_boundary are asymmetrical.
           (event::Id::Goal, event::Type::build(0.01, 5.5 / 56.0, 0.75))
        ]);
        return e;
    };
}

// Initialise the database.
pub fn initialise() {
    add_competition_data();

    // Creating the countries.
    let country_names: Vec<String> = io::get_countries_from_name_files();
    for name in country_names {
        Country::build_and_save(name);
    }
}

// Add competitions.
fn add_competition_data() {
    Competition::build_and_save(
        "Liiga",
        vec![
            Team::build_and_save("Blues"),
            Team::build_and_save("HIFK"),
            Team::build_and_save("HPK"),
            Team::build_and_save("Ilves"),
            Team::build_and_save("Jokerit"),
            Team::build_and_save("JYP"),
            Team::build_and_save("KalPa"),
            Team::build_and_save("Kärpät"),
            Team::build_and_save("Lukko"),
            Team::build_and_save("Pelican"),
            Team::build_and_save("SaiPa"),
            Team::build_and_save("Tappara"),
            Team::build_and_save("TPS"),
            Team::build_and_save("Ässät"),
        ],
        vec![
            Stage::build_and_save(
                "Regular Season",
                Some(RoundRobin::build(4, 0, 3, 2, 1, 1, 0)),
                None,
                match_event::Rules::build(3, 1200, 300, false),
                [9, 1],
                [4, 1]
            )
        ]
    );
}