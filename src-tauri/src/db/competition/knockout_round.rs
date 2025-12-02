use crate::logic::{competition::knockout_round::KnockoutRound, types::{CompetitionId, Db}};

impl KnockoutRound {
    pub async fn save(&self, db: &Db, comp_id: CompetitionId) {
        sqlx::query(
            "INSERT INTO KnockoutRoundFormat
            (comp_id, wins_required)
            VALUES ($1, $2)"
        ).bind(comp_id)
        .bind(self.wins_required)
        .execute(db).await.unwrap();
    }
}