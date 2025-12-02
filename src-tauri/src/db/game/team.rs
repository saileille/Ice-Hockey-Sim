use sqlx::{Row, sqlite::SqliteRow};

use crate::logic::{game::team::TeamGame, person::{contract::ContractRole, player::Player}, team::lineup::LineUp, types::{Db, GameId, TeamId}};

impl TeamGame  {
    pub fn custom_from_row(row: &SqliteRow, home_away: &str) -> sqlx::Result<Self> {
        Ok(Self {
            game_id: row.try_get("id")?,
            team_id: row.try_get(format!("{home_away}_id").as_str())?,
            lineup: row.try_get(format!("{home_away}_lineup").as_str())?,

            ..Default::default()
        })
    }

    pub async fn save(db: &Db, game_id: GameId, team_id: TeamId) {
        sqlx::query(
            "INSERT INTO TeamGame
            (game_id, team_id, lineup)
            VALUES ($1, $2, $3)"
        ).bind(game_id)
        .bind(team_id)
        .bind(LineUp::default())
        .execute(db).await.unwrap();
    }

    pub async fn overwrite(&mut self, db: &Db) {
        sqlx::query(
            "UPDATE TeamGame SET lineup = $1
            WHERE game_id = $2 AND team_id = $3"
        ).bind(&self.lineup)
        .bind(self.game_id)
        .bind(self.team_id)
        .execute(db).await.unwrap();

        for shot in self.shots.iter_mut() {
            shot.save(db).await;
        }
    }

    // Get the team name.
    pub async fn team_name(&self, db: &Db) -> String {
        sqlx::query_scalar(
            "SELECT full_name FROM Team
            WHERE id = $1"
        ).bind(self.team_id)
        .fetch_one(db).await.expect(format!(
            "id: {}", self.team_id
        ).as_str())
    }

    pub async fn team_seed(&self, db: &Db) -> u8 {
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

    // Get every player in the roster.
    pub async fn players(&self, db: &Db) -> Vec<Player> {
        sqlx::query_as(
            "SELECT Person.*, Player.ability, Player.position_id FROM Contract
            INNER JOIN Person ON Person.id = Contract.person_id
            INNER JOIN Player ON Player.person_id = Person.id
            WHERE Contract.team_id = $1
            AND Contract.is_signed = TRUE
            AND Contract.role = $2"
        ).bind(self.team_id)
        .bind(ContractRole::Player)
        .fetch_all(db).await.unwrap()
    }
}