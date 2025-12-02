mod lineup;

use serde_json::json;
use sqlx::{Row, sqlite::SqliteRow};
use time::Date;

use crate::logic::{person::{contract::ContractRole, manager::Manager, player::Player}, team::Team, time::AnnualWindow, types::{Db, TeamId}};

impl Team {
    pub fn custom_from_row(row: &SqliteRow, home_away: &str) -> sqlx::Result<Self> {
        Ok(Self {
            id: row.try_get(format!("{home_away}_id").as_str())?,
            full_name: row.try_get(format!("{home_away}_name").as_str())?,
            lineup: row.try_get(format!("{home_away}_lineup").as_str())?,
            primary_comp_id: row.try_get(format!("{home_away}_comp_id").as_str())?,
            player_needs: serde_json::from_value(row.try_get(format!("{home_away}_player_needs").as_str())?).unwrap(),
            actions_remaining: row.try_get(format!("{home_away}_actions_remaining").as_str())?,
        })
    }

    // Save the player needs to the database.
    pub async fn save_player_needs(&self, db: &Db) {
        sqlx::query(
            "UPDATE Team SET player_needs = $1
            WHERE id = $2"
        ).bind(json!(self.player_needs))
        .bind(self.id)
        .execute(db).await.unwrap();
    }

    pub async fn fetch_from_db(db: &Db, id: TeamId) -> Self {
        sqlx::query_as(
            "SELECT * FROM Team
            WHERE id = $1"
        ).bind(id)
        .fetch_one(db).await.unwrap()
    }

    // Fetch ALL teams from the database.
    pub async fn fetch_all(db: &Db) -> Vec<Self> {
        sqlx::query_as(
            "SELECT * FROM Team"
        ).fetch_all(db).await.unwrap()
    }

    // Update the Team to database.
    pub async fn save(&mut self, db: &Db) {
        self.id = sqlx::query_scalar(
            "INSERT INTO Team (full_name, lineup, primary_comp_id, player_needs, actions_remaining)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id"
        ).bind(self.full_name.as_str())
        .bind(&self.lineup)
        .bind(self.primary_comp_id)
        .bind(json!(self.player_needs))
        .bind(self.actions_remaining)
        .fetch_one(db).await.unwrap();
    }

    // Get the team's manager.
    pub async fn manager(&self, db: &Db) -> Option<Manager> {
        sqlx::query_as(
            "SELECT Person.*, Manager.is_human FROM Contract
            INNER JOIN Person ON Person.id = Contract.person_id
            INNER JOIN Manager ON Manager.person_id = Person.id
            WHERE Contract.team_id = $1
            AND Contract.is_signed = TRUE
            AND Contract.role = $2"
        ).bind(self.id)
        .bind(ContractRole::Manager)
        .fetch_optional(db).await.unwrap()
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
        ).bind(self.id)
        .bind(ContractRole::Player)
        .fetch_all(db).await.unwrap()
    }

    // Get the players in the roster, and those that are approached.
    pub async fn players_and_approached(&self, db: &Db) -> Vec<Player> {
        sqlx::query_as(
            "SELECT Person.*, Player.ability, Player.position_id FROM Contract
            INNER JOIN Person ON Person.id = Contract.person_id
            INNER JOIN Player ON Player.person_id = Person.id
            WHERE Contract.team_id = $1
            AND Contract.role = $2"
        ).bind(self.id)
        .bind(ContractRole::Player)
        .fetch_all(db).await.unwrap()
    }

    // Get the players to whom the team has offered contracts.
    pub async fn approached_players(&self, db: &Db) -> Vec<Player> {
        sqlx::query_as(
            "SELECT Person.*, Player.ability, Player.position_id FROM Contract
            INNER JOIN Person ON Person.id = Contract.person_id
            INNER JOIN Player ON Player.person_id = Person.id
            WHERE Contract.team_id = $1
            AND Contract.is_signed = FALSE
            AND Contract.role = $2"
        ).bind(self.id)
        .bind(ContractRole::Player)
        .fetch_all(db).await.unwrap()
    }

    pub async fn set_actions_remaining(&mut self, db: &Db, value: u8) {
        self.actions_remaining = value;
        sqlx::query(
            "UPDATE Team SET actions_remaining = $1
            WHERE id = $2"
        ).bind(self.actions_remaining)
        .bind(self.id)
        .execute(db).await.unwrap();
    }

    // Return whether this day is the season end date.
    pub async fn is_season_end_date(&self, db: &Db, today: Date) -> bool {
        let window: AnnualWindow = sqlx::query_scalar(
            "SELECT season_window FROM Competition
            WHERE id = $1"
        ).bind(self.primary_comp_id)
        .fetch_one(db).await.unwrap();

        return window.is_last_day(today);
    }
}