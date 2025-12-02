use crate::logic::{competition::round_robin::RoundRobin, types::{CompetitionId, Db}};

impl RoundRobin {
    pub async fn save(&self, db: &Db, comp_id: CompetitionId) {
        sqlx::query(
            "INSERT INTO RoundRobinFormat
            (comp_id, rounds, extra_matches, points_for_win, points_for_ot_win, points_for_draw, points_for_ot_loss, points_for_loss)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
        ).bind(comp_id)
        .bind(self.rounds)
        .bind(self.extra_matches)
        .bind(self.points_for_win)
        .bind(self.points_for_ot_win)
        .bind(self.points_for_draw)
        .bind(self.points_for_ot_loss)
        .bind(self.points_for_loss)
        .execute(db).await.unwrap();
    }
}