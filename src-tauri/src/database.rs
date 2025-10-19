// The game database.
use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex}
};
use rand::rng;
use time::{macros::date, Date};
use lazy_static::lazy_static;

use crate::{
    competition::{
        format::{self}, knockout_generator, season::{ranking::RankCriteria, Season}, CompConnection, Competition, Seed
    }, country::Country, event, io, match_event, person::{manager::Manager, player::{
        position::{Position, PositionId}, Player
    }}, team::Team, time::{AnnualDate, AnnualWindow}, types::{CompetitionId, CountryId, ManagerId, PlayerId, TeamId}
};

// The current date in the game.
pub static TODAY: LazyLock<Mutex<Date>> = LazyLock::new(|| Mutex::new(date!(2025-07-01)));

pub static COUNTRIES: LazyLock<Mutex<HashMap<CountryId, Country>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
pub static COMPETITIONS: LazyLock<Mutex<HashMap<CompetitionId, Competition>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

// Seasons are special in that they are stored in vectors by competition ID.
pub static SEASONS: LazyLock<Mutex<HashMap<CompetitionId, Vec<Season>>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

pub static TEAMS: LazyLock<Mutex<HashMap<TeamId, Team>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
pub static PLAYERS: LazyLock<Mutex<HashMap<PlayerId, Player>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
pub static MANAGERS: LazyLock<Mutex<HashMap<ManagerId, Manager>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

lazy_static! {
    pub static ref POSITIONS: HashMap<PositionId, Position> = {
         let p = HashMap::from([
            (PositionId::default(), Position::default()),
            (PositionId::Goalkeeper, Position::build(
                PositionId::Goalkeeper, "GK", 0
            )),
            (PositionId::LeftDefender, Position::build(
                PositionId::LeftDefender, "LD", 0
            )),
            (PositionId::RightDefender, Position::build(
                PositionId::RightDefender, "RD", 0
            )),
            (PositionId::LeftWinger, Position::build(
                PositionId::LeftWinger, "LW", 0
            )),
            (PositionId::Centre, Position::build(
                PositionId::Centre, "C", 0
            )),
            (PositionId::RightWinger, Position::build(
                PositionId::RightWinger, "RW", 0
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

    let comps = COMPETITIONS.lock().unwrap().clone();
    for comp in comps.values() {
        // Add parent IDs.
        comp.give_id_to_children_comps();

        // Set up seasons, starting from the top level.
        if comp.parent_comp_id == 0 {
            comp.setup_season(&mut Vec::new());
        }
    }

    // Creating the countries.
    let country_names = io::get_countries_from_name_files();
    for name in country_names.iter() {
        Country::build_and_save(name);
    }

    // Generate 50 players per team.
    let mut rng = rng();
    for _ in 0..TEAMS.lock().unwrap().len() * 50 {
        Player::build_and_save_random(&mut rng);
    }

    // Set up the teams.
    let mut teams = TEAMS.lock().unwrap().clone();
    for team in teams.values_mut() {
        team.setup(0, 0);
    }
}

// Add competitions.
fn add_competition_data() {
    // 1: Liiga
    Competition::build_and_save(
        "Liiga",
        vec![
            Team::build_and_save("Gestapojat"), // 1
            Team::build_and_save("Veto"),       // 2
            Team::build_and_save("Uupuneet"),   // 3
            Team::build_and_save("SantaClaus"), // 4
            Team::build_and_save("HardCore"),   // 5
            Team::build_and_save("Vauhti"),     // 6
            Team::build_and_save("Vimma"),      // 7
            Team::build_and_save("Kelarotat"),  // 8
            Team::build_and_save("Saappaat"),   // 9
            Team::build_and_save("Katiska"),    // 10
            Team::build_and_save("Turmio"),     // 11
            Team::build_and_save("Mahti"),      // 12
            Team::build_and_save("Merirosvot"), // 13
            Team::build_and_save("Sirkus"),     // 14
        ],
        AnnualWindow::build(
            AnnualDate::build(9, 1),
            AnnualDate::build(6, 1)
        ),
        Vec::new(),
        0,
        None,
        vec![RankCriteria::ChildCompRanking],
        vec![2, 3]
    );
    // 2: Liiga Regular Season.
    Competition::build_and_save(
        "Regular Season",
        Vec::new(),
        AnnualWindow::build(
            AnnualDate::build(9, 1),
            AnnualDate::build(4, 1)
        ),
        vec![CompConnection::build([1, 10], 3, Seed::GetFromPosition, false)],
        14,
        format::Format::build(
            Some(format::round_robin::RoundRobin::build(4, 0, 3, 2, 1, 1, 0)),
            None,
            match_event::Rules::build(3, 1200, 300, false)
        ),
        vec![
            RankCriteria::Points,
            RankCriteria::GoalDifference,
            RankCriteria::GoalsScored,
            RankCriteria::TotalWins,
            RankCriteria::RegularWins,
            RankCriteria::OvertimeWins,
            RankCriteria::Draws,
            RankCriteria::RegularLosses,
        ],
        Vec::new()
    );
    // 3: Liiga Playoffs.
    knockout_generator::build(
        "Playoffs",
        vec!["Säälit"],
        AnnualWindow::build(
            AnnualDate::build(4, 2),
            AnnualDate::build(6, 1)
        ),
        vec![match_event::Rules::build(3, 1200, 0, true)],
        vec![2, 4],
        vec![10],
        1,
        Vec::new(),
        vec![RankCriteria::Seed]
    );


}