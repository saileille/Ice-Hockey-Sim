pub mod position;
mod ai;


use rand::rngs::ThreadRng;
use serde_json::json;
use sqlx::{Row, FromRow, sqlite::SqliteRow};

use crate::{person::{Gender, attribute::{AttributeId, PersonAttribute}}, types::{Db, PersonId, TeamId}};
use super::Person;
use self::position::PositionId;

#[derive(Debug)]
#[derive(Default, Clone)]
pub struct Player {
    pub person: Person,
    pub ability: PersonAttribute,
    pub position_id: PositionId,
}

impl FromRow<'_, SqliteRow> for Player {
    fn from_row(row: &SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            person: Person::from_row(row)?,
            ability: PersonAttribute::build(AttributeId::General, row.try_get("ability")?),
            position_id: row.try_get("position_id")?,
        })
    }
}

// Basics.
impl Player {
    fn build(person: Person, position_id: PositionId) -> Self {
        Self {
            person: person,
            ability: PersonAttribute::build(AttributeId::General, 0),
            position_id: position_id,
            ..Default::default()
        }
    }

    // Create a player and store it in the database. Return a clone of the Player.
    pub async fn build_and_save(db: &Db, min_age: u8, max_age: u8) -> Self {
        let player = Self::create(db, min_age, max_age).await;
        player.save(db).await;
        return player;
    }

    // Just like build and save, but minimal arguments.
    pub async fn create(db: &Db, min_age: u8, max_age: u8) -> Self {
        let person = Person::create(db, min_age, max_age, Gender::Male).await;
        let position_id = PositionId::get_random();

        let mut player = Self::build(person, position_id);
        player.create_ability(db).await;

        return player;
    }

    // Create the ability of a player during its generation.
    // Simulate the player's training for every day of their life so far.
    async fn create_ability(&mut self, db: &Db) {
        let days = self.person.age_in_days(db).await;
        let mut rng = rand::rng();
        for i in 0..days {
            self.train(&mut rng, i);
        }
    }

    // Get a player from the database.
    pub async fn fetch_from_db(db: &Db, id: PersonId) -> Option<Self> {
        sqlx::query_as(
            "SELECT Person.*, Player.ability, Player.position_id FROM Person
            INNER JOIN Player ON Person.id = Player.person_id
            WHERE Person.id = $1"
        ).bind(id)
        .fetch_optional(db).await.unwrap()
    }

    // Get all active players.
    pub async fn fetch_active(db: &Db) -> Vec<Self> {
        sqlx::query_as(
            "SELECT Person.*, Player.ability, Player.position_id FROM Person
            INNER JOIN Player ON Person.id = Player.person_id
            WHERE Person.is_active = TRUE"
        ).fetch_all(db).await.unwrap()
    }

    // Save to database.
    pub async fn save(&self, db: &Db) {
        sqlx::query(
            "INSERT INTO Player
            (person_id, ability, position_id)
            VALUES ($1, $2, $3)"
        ).bind(self.person.id)
        .bind(self.ability.value)
        .bind(self.position_id)
        .execute(db).await.unwrap();
    }

    // Get a clone of the player's position abbreviation.
    async fn position_abbr(&self, db: &Db) -> String {
        sqlx::query_scalar(
            "SELECT abbreviation FROM Position
            WHERE id = $1"
        ).bind(self.position_id)
        .fetch_one(db).await.unwrap()
    }

    // Get all free agents from the database with given positions, which the given team has not approached yet.
    pub async fn free_agents_for_team(db: &Db, positions: Vec<&PositionId>, team_id: TeamId) -> Vec<Self> {
        let mut position_query = String::new();
        for position in positions {
            match position_query.is_empty() {
                true => position_query = "(".to_string(),
                false => position_query.push_str(" OR ")
            }

            position_query.push_str(format!("Player.position_id = {}", *position as u8).as_str());
        }

        position_query.push_str(")");

        sqlx::query_as(
            format!("SELECT Person.*, Player.ability, Player.position_id FROM Person
            INNER JOIN Player ON Player.person_id = Person.id
            WHERE Person.is_active = TRUE AND {position_query} AND
            Person.id NOT IN (
                SELECT person_id FROM Contract
                WHERE is_signed = TRUE
                OR team_id = $1
            )").as_str()
        ).bind(team_id)
        .fetch_all(db).await.unwrap()
    }

    // Get all free agents.
    async fn free_agents(db: &Db) -> Vec<Self> {
        sqlx::query_as(
            "SELECT Person.*, Player.ability, Player.position_id FROM Person
            INNER JOIN Player ON Player.person_id = Person.id
            WHERE Person.is_active = TRUE
            AND Person.id NOT IN (
                SELECT person_id FROM Contract
                WHERE is_signed = TRUE
            )"
        )
        .fetch_all(db).await.unwrap()
    }

    // Get all free agents in the game.
    pub async fn free_agents_package(db: &Db) -> Vec<serde_json::Value> {
        let mut players = Self::free_agents(db).await;

        players.sort_by(|a, b|
            b.ability.display().cmp(&a.ability.display())
            .then((a.position_id.clone() as u8).cmp(&(b.position_id.clone() as u8)))
            .then(a.person.surname.cmp(&b.person.surname))
            .then(a.person.forename.cmp(&b.person.forename))
        );

        let mut json = Vec::new();
        for player in players {
            json.push(player.package(db).await);
        }
        return json;
    }

    // Get relevant information of the player.
    pub async fn package(&self, db: &Db) -> serde_json::Value {
        json!({
            "person": self.person.package(db).await,
            "id": self.person.id,
            "position": self.position_abbr(db).await,
            "ability": self.ability.display(),
            "real_ability": self.ability.get(),
        })
    }

    // Get the position and the ID of the player.
    pub async fn roster_overview_package(&self, db: &Db, in_roster: bool) -> serde_json::Value {
        json!({
            "position": self.position_abbr(db).await,
            "id": self.person.id,
            "in_roster": in_roster,
        })
    }

    // The daily training of the player.
    pub async fn daily_training(&mut self, db: &Db) {
        let age = self.person.age_in_days(db).await;
        self.train(&mut rand::rng(), age);

        sqlx::query(
            "UPDATE Player SET ability = $1
            WHERE person_id = $2"
        ).bind(self.ability.value)
        .bind(self.person.id)
        .execute(db).await.unwrap();
    }

    // Do the training (also used in player generation).
    fn train(&mut self, rng: &mut ThreadRng, age_days: u16) {
        self.ability.update(age_days, rng);
    }
}