// The game database.
mod competition;
mod game;
mod person;

mod country;
mod team;
mod time;

use std::{collections::HashMap, path::Path};
use rand::rngs::ThreadRng;
use sqlx::{Sqlite, migrate::MigrateDatabase, sqlite::SqlitePoolOptions};
use tauri::{AppHandle, Manager as TauriManager, path::BaseDirectory};
use ::time::{macros::date, Date};
use lazy_static::lazy_static;

use crate::logic::{app_data::{AppData, CountryWeights, Directories}, competition::{Competition, Seed, comp_connection::CompConnection, knockout_generator, round_robin::RoundRobin as RoundRobinFormat, season::ranking::RankCriteria}, country::Country, event, game as match_event, io::{get_countries_from_name_files, remove_db}, person::{attribute::{Attribute, AttributeId}, player::Player}, team::Team, time::{AnnualDate, AnnualWindow}, types::Db};

// Get the current date.
pub async fn get_today(db: &Db) -> Date {
    sqlx::query_scalar(
        "SELECT value_data FROM KeyValue
        WHERE key_name = 'today'"
    ).fetch_one(db).await.unwrap()
}

// Continue to the next day.
pub async fn next_day(db: &Db, today: Date) {
    sqlx::query(
        "UPDATE KeyValue SET value_data = $1
        WHERE key_name = 'today'"
    ).bind(today.next_day().unwrap())
    .execute(db).await.unwrap();
}

lazy_static! {
    pub static ref ATTRIBUTES: HashMap<AttributeId, Attribute> = {
         HashMap::from([
             (AttributeId::Defending, Attribute::build(
                AttributeId::Defending, 0, 0
            )),
            (AttributeId::Shooting, Attribute::build(
                AttributeId::Shooting, 0, 0
            )),
            (AttributeId::Passing, Attribute::build(
                AttributeId::Passing, 0, 0
            )),
            (AttributeId::Faceoffs, Attribute::build(
                AttributeId::Faceoffs, 0, 0
            )),
            (AttributeId::General, Attribute::build(
                AttributeId::General, 6, 26
            )),
        ])
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

pub async fn setup(dir: &Path) -> Db {
    // Database in resource folder.
    let canonised = dunce::canonicalize(dir).unwrap();
    let path = canonised.to_str().unwrap();
    Sqlite::create_database(format!("sqlite://{path}/db.db?mode=rwc").as_str()).await.unwrap();
    let db = SqlitePoolOptions::new().connect(format!("sqlite://{path}/db.db").as_str()).await.unwrap();

    // Database in memory
    // Sqlite::create_database(format!("sqlite::memory:").as_str()).await.unwrap();
    // let db = SqlitePoolOptions::new().connect(format!("sqlite::memory:").as_str()).await.unwrap();

    // Database in src folder (testing only).
    // Sqlite::create_database(format!("sqlite://data/db.db?mode=rwc").as_str()).await.unwrap();
    // let db = SqlitePoolOptions::new().connect(format!("sqlite://data/db.db").as_str()).await.unwrap();

    sqlx::migrate!("sql/migrations").run(&db).await.unwrap();

    return db;
}

// Initialise the database.
pub async fn initialise(handle: &AppHandle) -> AppData {
    let mut data = create_dir_paths(handle).await;

    // Creating the start date and saving it to the database.
    let today = date!(2025-07-01);
    sqlx::query(
        "INSERT INTO KeyValue (key_name, value_data)
        VALUES ('today', $1)"
    ).bind(today)
    .execute(&data.db).await.unwrap();

    let mut rng = rand::rng();

    add_competitions(&data.db, &mut rng, today).await;
    initialise_seasons(&data.db).await;
    create_countries(&data).await;

    data.country_weights = CountryWeights::build(&data.db).await;

    let teams = Team::fetch_all(&data.db).await;
    generate_free_agent_players(&data, today, &teams).await;
    setup_teams(&data, today, teams).await;

    return data
}

// Create the resource directory paths.
async fn create_dir_paths(handle: &AppHandle) -> AppData {
    let data_dir = handle.path().resolve("data/", BaseDirectory::Resource).unwrap();
    let people_name_dir = data_dir.join("names/");
    let flag_dir = data_dir.join("flags/");

    let directories = Directories {
        names: people_name_dir.to_str().unwrap().to_string(),
        flags: flag_dir.to_str().unwrap().to_string(),
        db: data_dir.to_str().unwrap().to_string(),
    };

    remove_db(&directories);

    let db = setup(&data_dir.as_path()).await;
    return AppData::build(db, directories);
}

// Add competitions.
// NOTE: Season window of the parent competition MUST go at least one day past the last day of the last stage.
// Otherwise some contracts might expire before the last match day is played.
async fn add_competitions(db: &Db, rng: &mut ThreadRng, today: Date) {
    let now = std::time::Instant::now();

    Competition::build_and_save(
        db, today,
        "PHL",
        vec![
            Team::build("Ruiske"),     // 1
            Team::build("Atomi"),      // 2
            Team::build("Uupuneet"),   // 3
            Team::build("SantaClaus"), // 4
            Team::build("HardCore"),   // 5
            Team::build("Ikirouta"),   // 6
            Team::build("Kelarotat"),  // 7
            Team::build("Vety"),       // 8
            Team::build("Saappaat"),   // 9
            Team::build("Siat"),       // 10
            Team::build("Turmio"),     // 11
            Team::build("Sirkus"),     // 12
            Team::build("Polkka"),     // 13
            Team::build("Teurastus"),  // 14
        ],
        AnnualWindow::build(
            AnnualDate::build(9, 1),
            AnnualDate::build(6, 1)
        ),
        Vec::new(),
        0,
        None,
        None,
        None,
        vec![RankCriteria::ChildCompRanking],
        vec![
            Competition::build_and_save(
                db, today,
                "Regular Season",
                Vec::new(),
                AnnualWindow::build(
                    AnnualDate::build(9, 1),
                    AnnualDate::build(3, 31)
                ),
                Vec::new(),
                14,
                Some(match_event::Rules::build_and_save(db, 3, 1200, 300, false).await),
                Some(RoundRobinFormat::build_and_save(db, 4, 0, 3, 2, 1, 1, 0).await),
                None,
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
                Vec::new(),
            ).await,
            knockout_generator::build(
                db, rng, today,
                "Playoffs",
                vec!["Pity Round"],
                AnnualWindow::build(
                    AnnualDate::build(4, 1),
                    AnnualDate::build(5, 31)
                ),
                vec![match_event::Rules::build_and_save(db, 3, 1200, 0, true).await],
                vec![2, 4],
                vec![10],
                1,
                vec![CompConnection::build(1, 1, 10, Seed::GetFromPosition, false)],
                vec![RankCriteria::Seed],
            ).await
        ],
    ).await;

    println!("Added competitions in {:.2?}", now.elapsed());
}

// Set up seasons for the first time, starting from the parent competitions and going down the hierarchy.
async fn initialise_seasons(db: &Db) {
    let now = std::time::Instant::now();
    let comps = Competition::fetch_parents(db).await;
    for comp in comps {
        comp.setup_season(db, &mut Vec::new()).await;
    }
    println!("Initialised seasons in {:.2?}", now.elapsed());
}

async fn create_countries(data: &AppData) {
    let now = std::time::Instant::now();
    let country_names = get_countries_from_name_files(&data.directories);
    for name in country_names.iter() {
        Country::build_and_save(&data.directories, &data.db, name).await;
    }
    println!("Created countries in {:.2?}", now.elapsed());
}

// Generate a certain amount of free agents per team.
async fn generate_free_agent_players(data: &AppData, today: Date, teams: &[Team]) {
    let now = std::time::Instant::now();
    for _ in 0..teams.len() * 50 {
        Player::build_and_save(data, today, 16, 35).await;
    }
    println!("Generated free agents in {:.2?}", now.elapsed());
}

async fn setup_teams(data: &AppData, today: Date, teams: Vec<Team>) {
    let now = std::time::Instant::now();
    for mut team in teams.into_iter() {
        team.setup(data, today).await;
    }
    println!("Set up teams in {:.2?}", now.elapsed());
}