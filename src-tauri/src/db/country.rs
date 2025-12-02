use serde_json::json;

use crate::logic::{country::Country, types::{CountryId, Db}};

impl Country {
    // Get a Country from the database.
    pub async fn fetch_from_db(db: &Db, id: CountryId) -> Self {
        let country = sqlx::query_as(
            "SELECT * FROM Country WHERE id = $1"
        ).bind(id)
        .fetch_one(db).await.unwrap();

        return country;
    }

    // Fetch ALL from the database.
    pub async fn fetch_all(db: &Db) -> Vec<Self> {
        sqlx::query_as(
            "SELECT * FROM Country"
        )
        .fetch_all(db).await.unwrap()
    }

    // Save the Country to database and return the ID.
    pub async fn save(&mut self, db: &Db) {
        sqlx::query(
            "INSERT INTO Country
            (country_name, names, flag_path)
            VALUES ($1, $2, $3)"
        ).bind(&self.name)
        .bind(json!(self.names))
        .bind(&self.flag_path)
        .execute(db).await.unwrap();
    }
}