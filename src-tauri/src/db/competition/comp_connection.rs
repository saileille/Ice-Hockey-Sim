use crate::logic::{competition::{Competition, comp_connection::CompConnection}, types::{CompetitionId, Db}};

impl CompConnection {
    pub async fn save(&self, db: &Db, destination_id: CompetitionId) {
        sqlx::query(
            "INSERT INTO CompConnection
            (origin_id, destination_id, highest_position, lowest_position, team_seeds, stats_carry_over)
            VALUES ($1, $2, $3, $4, $5, $6)"
        ).bind(self.origin_id)
        .bind(destination_id)
        .bind(self.highest_position)
        .bind(self.lowest_position)
        .bind(self.team_seeds)
        .bind(self.stats_carry_over)
        .execute(db).await.unwrap();
    }

    pub async fn destination(&self, db: &Db) -> Competition {
        sqlx::query_as(
            "SELECT * FROM Competition WHERE id = $1"
        ).bind(self.destination_id)
        .fetch_one(db).await.unwrap()
    }
}