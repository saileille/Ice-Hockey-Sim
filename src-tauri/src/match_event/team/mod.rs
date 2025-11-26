pub mod cache;

use std::num::NonZero;

use serde_json::json;
use sqlx::FromRow;

use crate::{match_event::event::Shot, team::lineup::LineUp, types::{Db, GameId, TeamId}};

#[derive(Debug)]
#[derive(Default, Clone)]
#[derive(FromRow)]
pub struct TeamGame {
    game_id: GameId,
    pub team_id: TeamId,
    #[sqlx(json)]
    pub shots: Vec<Shot>,
    pub lineup: LineUp,
}

// Basics.
impl TeamGame {
    pub fn build(game_id: GameId, team_id: TeamId) -> Self {
        Self {
            game_id,
            team_id,

            ..Default::default()
        }
    }

    pub async fn save(&self, db: &Db) {
        sqlx::query(
            "INSERT INTO TeamGame
            (game_id, team_id, shots, lineup)
            VALUES ($1, $2, $3, $4)"
        ).bind(NonZero::new(self.game_id).unwrap())
        .bind(NonZero::new(self.team_id).unwrap())
        .bind(json!(self.shots))
        .bind(&self.lineup)
        .execute(db).await.unwrap();
    }

    pub async fn overwrite(&self, db: &Db) {
        sqlx::query(
            "UPDATE TeamGame SET shots = $1
            WHERE game_id = $2 AND team_id = $3"
        ).bind(json!(self.shots))
        .bind(NonZero::new(self.game_id).unwrap())
        .bind(NonZero::new(self.team_id).unwrap())
        .execute(db).await.unwrap();
    }

    async fn fetch_from_db(db: &Db, game_id: GameId, team_id: TeamId) -> Self {
        sqlx::query_as(
            "SELECT * FROM TeamGame
            WHERE game_id = $1 AND team_id = $2"
        ).bind(game_id)
        .bind(team_id)
        .fetch_one(db).await.unwrap()
    }

    // Get the team name.
    async fn team_name(&self, db: &Db) -> String {
        sqlx::query_scalar(
            "SELECT full_name FROM Team
            WHERE id = $1"
        ).bind(self.team_id)
        .fetch_one(db).await.expect(format!(
            "id: {}", self.team_id
        ).as_str())
    }

    async fn team_seed(&self, db: &Db) -> u8 {
        sqlx::query_scalar(
            "SELECT seed FROM TeamSeason
            INNER JOIN Season
            ON Season.id = TeamSeason.season_id
            WHERE TeamSeason.team_id = $1
            AND Season.id = (
                SELECT season_id FROM Game
                WHERE id = $2
            )"
        ).bind(self.team_id)
        .bind(self.game_id)
        .fetch_one(db).await.unwrap()
    }

    pub async fn comp_screen_package(&self, db: &Db) -> serde_json::Value {
        json!({
            "id": self.team_id,
            "name": self.team_name(db).await,
            "seed": self.team_seed(db).await,
            "goals": self.goals()
        })
    }
}

// Functional.
impl TeamGame {
    pub fn goals(&self) -> u16 {
        let mut goal_counter = 0;
        for shot in self.shots.iter() {
            if shot.is_goal { goal_counter += 1; }
        }
        return goal_counter;
    }
}