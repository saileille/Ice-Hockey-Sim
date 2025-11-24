pub mod cache;

use serde_json::json;
use sqlx::FromRow;

use crate::{match_event::event::Shot, team::lineup::LineUp, types::{Db, GameId, TeamId}};

#[derive(Debug)]
#[derive(Default, Clone)]
#[derive(FromRow)]
pub struct TeamGame {
    _game_id: GameId,
    pub team_id: TeamId,
    #[sqlx(json)]
    pub shots: Vec<Shot>,
    pub lineup: LineUp,
}

// Basics.
impl TeamGame {
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
            "SELECT team_name FROM Team
            WHERE id = $1"
        ).bind(self.team_id)
        .fetch_one(db).await.unwrap()
    }

    async fn team_seed(&self, db: &Db) -> u8 {
        sqlx::query_scalar(
            "SELECT seed FROM TeamSeason
            INNER JOIN Season
            ON Season.id = TeamSeason.season_id
            INNER JOIN Game
            ON Game.season_id = Season.id
            WHERE TeamSeason.team_id = $1
            AND Game.id = $2"
        ).bind(self.team_id)
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