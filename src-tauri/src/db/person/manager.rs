use sqlx::{FromRow, Row, sqlite::SqliteRow};

use crate::logic::{person::{Person, manager::Manager}, types::Db};

impl FromRow<'_, SqliteRow> for Manager {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        Ok(Self {
            person: Person::from_row(row)?,
            is_human: row.try_get("is_human")?,
        })
    }
}

impl Manager {
    // Get all active managers.
    pub async fn fetch_active(db: &Db) -> Vec<Self> {
        sqlx::query_as(
            "SELECT Person.*, Manager.is_human FROM Person
            INNER JOIN Manager ON Person.id = Manager.person_id
            WHERE Person.is_active = TRUE"
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
}