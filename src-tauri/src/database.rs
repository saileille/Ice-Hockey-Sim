// The game database.
use std::{collections::HashMap, sync::{LazyLock, Mutex}};
use lazy_static::lazy_static;

use crate::custom_types;

use crate::team::Team;
use crate::event;
use crate::person::player::{Player, position::{Position, PositionId}};
use crate::country::Country;
use crate::competition::{Competition, stage::{Stage, rules}};
use crate::match_event;
use crate::match_event::{Game};
use crate::io;

pub static COUNTRIES: LazyLock<Mutex<HashMap<custom_types::CountryId, Country>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

pub static COMPETITIONS: LazyLock<Mutex<HashMap<custom_types::CompetitionId, Competition>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
pub static STAGES: LazyLock<Mutex<HashMap<custom_types::StageId, Stage>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
pub static GAMES: LazyLock<Mutex<HashMap<custom_types::GameId, Game>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

pub static TEAMS: LazyLock<Mutex<HashMap<custom_types::TeamId, Team>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

pub static PLAYERS: LazyLock<Mutex<HashMap<custom_types::PlayerId, Player>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

lazy_static! {
    pub static ref POSITIONS: HashMap<PositionId, Position> = {
         let p = HashMap::from([
            (PositionId::default(), Position::default()),
            (PositionId::Goalkeeper, Position::new(
                PositionId::Goalkeeper, 0
            )),
            (PositionId::Defender, Position::new(
                PositionId::Defender, 0
            )),
            (PositionId::LeftWinger, Position::new(
                PositionId::LeftWinger, 0
            )),
            (PositionId::Centre, Position::new(
                PositionId::Centre, 0
            )),
            (PositionId::RightWinger, Position::new(
                PositionId::RightWinger, 0
            )),
        ]);
        return p;
    };

    pub static ref EVENT_TYPES: HashMap<event::Id, event::Type> = {
        let e = HashMap::from([
            // Chance of home team getting the puck. Failure means it goes to away team.
            (event::Id::PuckPossessionChange, event::Type::new(0.1, 0.5, 0.9)),

            // Chance of attacking team to get a shot at the goal.
            // Minimum chance is 10 times as low as the equilibrium, maximum chance is 10 times as high.
            (event::Id::ShotAtGoal, event::Type::new(5.6 / 3600.0, 56.0 / 3600.0, 560.0 / 3600.0)),

            // Chance of a shot going in goal.
            // NOTE: min_boundary and max_boundary are asymmetrical.
           (event::Id::Goal, event::Type::new(0.01, 5.5 / 56.0, 0.75)) 
        ]);
        return e;
    };

}

// Initialise the database.
pub fn initialise() {
    // Adding default data in...
    add_default_data();

    add_competition_data();

    // Creating the countries.
    let country_names: Vec<String> = io::get_countries_from_name_files();
    for name in country_names {
        Country::build_and_save(name);
    }
}

// Add the default values for each part of the database.
fn add_default_data() {
    PLAYERS.lock()
        .expect("something went wrong when trying to insert default Player to PLAYERS")
        .insert(0, Player::default());
    
    TEAMS.lock()
        .expect("something went wrong when trying to insert default Team to TEAMS")
        .insert(0, Team::default());
    
    COUNTRIES.lock()
        .expect("something went wrong when trying to insert default Country to COUNTRIES")
        .insert(0, Country::default());
    
    COMPETITIONS.lock()
        .expect("something went wrong when trying to insert default Competition to COMPETITIONS")
        .insert(0, Competition::default());
    
    STAGES.lock()
        .expect("something went wrong when trying to insert default Stage to STAGES")
        .insert(0, Stage::default());
    
    GAMES.lock()
        .expect("something went wrong when trying to insert default Game to GAMES")
        .insert(0, Game::default());
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
            Team::build_and_save("Pelicans"),
            Team::build_and_save("SaiPa"),
            Team::build_and_save("Tappara"),
            Team::build_and_save("TPS"),
            Team::build_and_save("Ässät"),
        ],
        vec![
            vec![
                Stage::build_and_save(
                    "Regular Season",
                    rules::RoundRobin::build(4, 0),
                    match_event::Rules::build(3, 1200, 300, false),
                )
            ]
        ]
    );
}