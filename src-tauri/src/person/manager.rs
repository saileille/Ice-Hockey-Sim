// This is what the player (user) is!

use serde_json::json;
use sqlx::{Row, FromRow, sqlite::SqliteRow};

use crate::{person::{Gender, Person}, types::{CountryId, Db}};

#[derive(Default, Clone)]
pub struct Manager {
    pub person: Person,
    pub is_human: bool,
}

impl FromRow<'_, SqliteRow> for Manager {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        Ok(Self {
            person: Person::from_row(row)?,
            is_human: row.try_get("is_human")?,
        })
    }
}

impl Manager {
    // Build a manager.
    fn build(person: Person, is_human: bool) -> Self {
        Self {
            person,
            is_human,
        }
    }

    // Create a manager and store it in the database. Return a clone of the Manager.
    async fn build_and_save(db: &Db, person: Person, is_human: bool) -> Self {
        let manager = Self::build(person, is_human);
        manager.save(db).await;
        return manager;
    }

    // Build a random manager.
    pub async fn build_and_save_random(db: &Db, country_weights: &[(CountryId, u32)], total_weight: u32, is_human: bool) -> Self {
        let person = Person::create(db, country_weights, total_weight, 30, 60, Gender::Male).await;
        return Self::build_and_save(db, person, is_human).await;
    }

    // Get all active managers.
    pub async fn fetch_active(db: &Db) -> Vec<Self> {
        sqlx::query_as(
            "SELECT Person.*, Manager.is_human FROM Person
            INNER JOIN Manager ON Person.id = Manager.person_id
            WHERE Person.is_active"
        ).fetch_all(db).await.unwrap()
    }

    // Update the manager to database.
    pub async fn save(&self, db: &Db) {
        sqlx::query(
            "INSERT INTO Manager
            (person_id, is_human)
            VALUES ($1, $2)"
        ).bind(self.person.id)
        .bind(self.is_human)
        .execute(db).await.unwrap();
    }

    // Get the human managers.
    pub async fn humans(db: &Db) -> Vec<Self> {
        sqlx::query_as(
            "SELECT Manager.is_human, Person.* FROM Manager
            INNER JOIN Person
            ON Manager.person_id = Person.id
            WHERE Manager.is_human = TRUE"
        )
        .fetch_all(db).await.unwrap()
    }

    // Get relevant information to the team screen.
    pub async fn team_screen_package(&self, db: &Db) -> serde_json::Value {
        json!({
            "person": self.person.package(db).await
        })
    }

    // Get a package of information that is used in game interaction.
    // For human players only.
    pub async fn package(&self, db: &Db) -> serde_json::Value {
        json!({
            "team": match self.person.contract(db).await {
                Some(contract) => Some(contract.team(db).await.manager_package(db).await),
                _ => None
            }
        })
    }
}