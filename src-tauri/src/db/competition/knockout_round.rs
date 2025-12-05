use crate::logic::{competition::knockout_round::KnockoutRound, types::{CompetitionId, Db}};

impl KnockoutRound {
    pub async fn save(&mut self, db: &Db) {
        self.id = sqlx::query_scalar(
            "INSERT INTO KnockoutRoundFormat
            (wins_required)
            VALUES ($1)
            RETURNING id"
        ).bind(self.wins_required)
        .fetch_one(db).await.unwrap();
    }
}