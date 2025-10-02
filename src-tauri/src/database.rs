// The game database.
use std::{collections::HashMap, sync::{LazyLock, Mutex}};
use lazy_static::lazy_static;

use crate::team::Team;
use crate::event;
use crate::person::player::{Player, position::{Position, PositionId}};
use crate::country::Country;
use crate::io;

pub static TEAMS: LazyLock<Mutex<HashMap<usize, Team>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
pub static PLAYERS: LazyLock<Mutex<HashMap<usize, Player>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
pub static COUNTRIES: LazyLock<Mutex<HashMap<usize, Country>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

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
    PLAYERS.lock()
        .expect("something went wrong when trying to insert default Player to PLAYERS")
        .insert(0, Player::default());
    
    TEAMS.lock()
        .expect("something went wrong when trying to insert default Team to TEAMS")
        .insert(0, Team::default());
    
    COUNTRIES.lock()
        .expect("something went wrong when trying to insert default Country to COUNTRIES")
        .insert(0, Country::default());

    let country_names: Vec<String> = io::get_countries_from_name_files();
    for name in country_names {
        Country::create_and_save(name);
    }
}