use crate::logic::{competition::season::team::TeamSeason, types::Db};

impl TeamSeason {
    pub async fn save(&self, db: &Db) {
        sqlx::query(
            "INSERT INTO TeamSeason
            (team_id, season_id, seed, ranking, regular_wins, ot_wins, draws, ot_losses, regular_losses, goals_scored, goals_conceded)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"
        ).bind(self.team_id)
        .bind(self.season_id)
        .bind(self.seed)
        .bind(self.rank)
        .bind(self.regular_wins)
        .bind(self.ot_wins)
        .bind(self.draws)
        .bind(self.ot_losses)
        .bind(self.regular_losses)
        .bind(self.goals_scored)
        .bind(self.goals_conceded)
        .execute(db).await.unwrap();
    }

    pub async fn team_name(&self, db: &Db) -> String {
        sqlx::query_scalar(
            "SELECT full_name FROM Team WHERE id = $1"
        ).bind(self.team_id)
        .fetch_one(db).await.unwrap()
    }

    // Update the team data after a match.
    pub async fn update_and_save(&mut self, db: &Db, game_data: &TeamSeason) {
        self.update(game_data);

        sqlx::query(
            "UPDATE TeamSeason SET
            regular_wins = $1, ot_wins = $2, draws = $3,
            ot_losses = $4, regular_losses = $5,
            goals_scored = $6, goals_conceded = $7
            WHERE team_id = $8 AND season_id = $9"
        ).bind(self.regular_wins)
        .bind(self.ot_wins)
        .bind(self.draws)
        .bind(self.ot_losses)
        .bind(self.regular_losses)
        .bind(self.goals_scored)
        .bind(self.goals_conceded)
        .bind(self.team_id)
        .bind(self.season_id)
        .execute(db).await.unwrap();
    }
}