mod contract;
mod manager;
mod player;

use crate::logic::{person::{Person, contract::Contract}, types::Db};

impl Person {
    pub async fn save(&mut self, db: &Db) {
        self.id = sqlx::query_scalar(
            "INSERT INTO Person
            (forename, surname, gender, country_id, birthday, is_active)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id"
        ).bind(self.forename.as_str())
        .bind(self.surname.as_str())
        .bind(self.gender)
        .bind(self.country_id)
        .bind(self.birthday)
        .bind(self.is_active)
        .fetch_one(db).await.unwrap();
    }

    // Change the person entry in the database to inactive.
    // Contracts are revoked later.
    pub async fn retire(&self, db: &Db) {
        sqlx::query(
            "UPDATE Person SET is_active = FALSE WHERE id = $1"
        ).bind(self.id)
        .execute(db).await.unwrap();
    }

    // Get the contract of the person.
    pub async fn contract(&self, db: &Db) -> Option<Contract> {
        sqlx::query_as(
            "SELECT * FROM Contract
            WHERE person_id = $1 AND is_signed = TRUE"
        ).bind(self.id)
        .fetch_optional(db).await.unwrap()
    }

    pub async fn contract_offers(&self, db: &Db) -> Vec<Contract> {
        sqlx::query_as(
            "SELECT * FROM Contract
            WHERE person_id = $1 AND is_signed = FALSE"
        ).bind(self.id)
        .fetch_all(db).await.unwrap()
    }

    // Get the amount of contract offers the person has.
    pub async fn no_of_offers(&self, db: &Db) -> u8 {
        sqlx::query_scalar(
            "SELECT COUNT(*) FROM Contract
            WHERE person_id = $1 AND is_signed = FALSE"
        ).bind(self.id)
        .fetch_one(db).await.unwrap()
    }
}