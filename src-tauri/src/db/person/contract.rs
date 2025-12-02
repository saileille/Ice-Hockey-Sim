use time::Date;

use crate::logic::{person::contract::Contract, team::Team, types::Db};

impl Contract {
    pub async fn save_new(&self, db: &Db) {
        sqlx::query(
            "INSERT INTO Contract
            (person_id, team_id, begin_date, end_date, role, is_signed)
            VALUES ($1, $2, $3, $4, $5, $6)"
        ).bind(self.person_id)
        .bind(self.team_id)
        .bind(self.start_date)
        .bind(self.end_date)
        .bind(self.role)
        .bind(self.is_signed)
        .execute(db).await.unwrap();
    }

    // Delete all expired contracts, and contracts of retired people.
    // Called AFTER day has been changed in the continue game function!
    pub async fn delete_expired_and_retired(db: &Db) {
        sqlx::query(
            "DELETE FROM Contract
            WHERE unixepoch(end_date) < (
                SELECT unixepoch(value_data) FROM KeyValue
                WHERE key_name = 'today'
            ) OR (
				SELECT is_active FROM Person
				WHERE id = person_id
			) = FALSE"
        ).execute(db).await.unwrap();
    }

    // Change this contract from unsigned to signed and set the start date to today.
    pub async fn sign(&self, db: &Db, today: Date) {
        sqlx::query(
            "UPDATE Contract
            SET is_signed = TRUE, begin_date = $1
            WHERE person_id = $2 AND team_id = $3"
        ).bind(today)
        .bind(self.person_id)
        .bind(self.team_id)
        .execute(db).await.unwrap();
    }

    // Get the team of the contract.
    pub async fn team(&self, db: &Db) -> Team {
        sqlx::query_as(
            "SELECT * FROM Team WHERE id = $1"
        ).bind(self.team_id)
        .fetch_one(db).await.unwrap()
    }
}