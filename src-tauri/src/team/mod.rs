pub mod lineup;
pub mod ai;


use std::num::NonZero;

use serde_json::json;
use sqlx::FromRow;
use time::macros::date;
use crate::{
    database, person::{Contract, ContractRole, manager::Manager, player::
        Player
    }, team::ai::PlayerNeed, time::AnnualWindow, types::{
        CompetitionId, CountryId, Db, PersonId, TeamId
    }
};
use self::lineup::LineUp;

#[derive(Debug)]
#[derive(Default, Clone)]
#[derive(FromRow)]
pub struct Team {
    pub id: TeamId,
    pub full_name: String,
    #[sqlx(json)]
    pub lineup: LineUp,
    pub primary_comp_id: CompetitionId,

    // Player-acquisition related.
    #[sqlx(json)]
    pub player_needs: Vec<PlayerNeed>,

    // Actions.
    pub actions_remaining: u8
}

// Basics.
impl Team {
    // Get the amount of teams in the database.
    async fn no_of_teams(db: &Db) -> TeamId {
        match sqlx::query_scalar(
            "SELECT max(id) FROM Team"
        ).fetch_one(db).await.unwrap() {
            Some(n) => n,
            _ => 0
        }
    }

    pub fn build(name: &str) -> Self {
        Self {
            full_name: name.to_string(),
            ..Default::default()
        }
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

    // Get the next ID to use.
    async fn next_id(db: &Db) -> TeamId {
        Self::no_of_teams(db).await + 1
    }

    // Give an ID to the team.
    pub async fn give_id(&mut self, db: &Db) {
        self.id = Self::next_id(db).await;
    }

    // Update the Team to database.
    pub async fn save(&self, db: &Db) {
        sqlx::query(
            "INSERT INTO Team (id, full_name, lineup, primary_comp_id, player_needs, actions_remaining)
            VALUES ($1, $2, $3, $4, $5, $6)"
        ).bind(NonZero::new(self.id).unwrap())
        .bind(self.full_name.as_str())
        .bind(&self.lineup)
        .bind(NonZero::new(self.primary_comp_id).unwrap())
        .bind(json!(self.player_needs))
        .bind(self.actions_remaining)
        .execute(db).await.unwrap();
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
    async fn players(&self, db: &Db) -> Vec<Player> {
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
    async fn players_and_approached(&self, db: &Db) -> Vec<Player> {
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
    async fn approached_players(&self, db: &Db) -> Vec<Player> {
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

    // Get info for a team screen in JSON.
    pub async fn team_screen_package(&self, db: &Db) -> serde_json::Value {
        let mut players = self.players_and_approached(db).await;
        players.sort_by(|a, b| (a.position_id.clone() as u8).cmp(&(b.position_id.clone() as u8)).then(b.ability.display().cmp(&a.ability.display())));

        let mut json_players = Vec::new();
        for player in players {
            json_players.push(player.package(db).await);
        }

        json!({
            "id": self.id,
            "name": self.full_name,
            "manager": match self.manager(db).await {
                Some(manager) => Some(manager.team_screen_package(db).await),
                _ => None
            },
            "players": json_players
        })
    }

    // Get relevant team info for a contract.
    pub fn contract_package(&self) -> serde_json::Value {
        json!({
            "id": self.id,
            "name": self.full_name
        })
    }

    // Get relevant info of the team for a human manager.
    pub async fn manager_package(&self, db: &Db) -> serde_json::Value {
        let approached_players = self.approached_players(db).await;
        let approached_ids: Vec<PersonId> = approached_players.iter().map(|a| a.person.id).collect();
        json!({
            "id": self.id,
            "actions_remaining": self.actions_remaining,
            "roster_overview": self.roster_overview_package(db, approached_players).await,
            "approached_players": approached_ids,
        })
    }

    // Get a roster overview of the team.
    async fn roster_overview_package(&self, db: &Db, approached: Vec<Player>) -> Vec<serde_json::Value> {
        let players = self.players(db).await;
        let mut overview = Vec::new();
        for player in players {
            overview.push(player.roster_overview_package(db, true).await);
        }

        for player in approached {
            overview.push(player.roster_overview_package(db, false).await);
        }
        return overview;
    }
}

impl Team {
    // Build a lineup for the team from its roster and save to database.
    pub async fn auto_build_lineup(&mut self, db: &Db) {
        self.lineup.clear();

        let mut players = self.players(db).await;
        players.sort_by(|a, b| b.ability.display().cmp(&a.ability.display()));

        self.lineup.auto_add(players);
        self.lineup.save(self.id, db).await;
    }

    async fn set_actions_remaining(&mut self, db: &Db, value: u8) {
        self.actions_remaining = value;
        sqlx::query(
            "UPDATE Team SET actions_remaining = $1
            WHERE id = $2"
        ).bind(self.actions_remaining)
        .bind(self.id)
        .execute(db).await.unwrap();
    }

    async fn remove_action(&mut self, db: &Db) {
        self.set_actions_remaining(db, self.actions_remaining - 1).await;
    }

    // Give the team its full actions back.
    // Action value could depend on quantity and quality of team staff?
    pub async fn return_actions_to_full(&mut self, db: &Db) {
        self.set_actions_remaining(db, 1).await;
    }

    // Return whether this day is the season end date.
    async fn is_season_end_date(&self, db: &Db) -> bool {
        let window: AnnualWindow = sqlx::query_scalar(
            "SELECT season_window FROM Competition
            WHERE id = $1"
        ).bind(self.primary_comp_id)
        .fetch_one(db).await.unwrap();

        return window.is_last_day(db).await;
    }
}

impl Team {
    // Create a manager out of thin air.
    async fn create_manager(&mut self, db: &Db, country_weights: &[(CountryId, u32)], total_weight: u32) {
        let manager = Manager::build_and_save_random(db, country_weights, total_weight, false).await;
        Contract::build_and_save(db, manager.person.id, self.id, database::get_today(db).await, date!(2125-06-01), ContractRole::Manager, true).await;
    }

    // Set up the team when initialising a game.
    pub async fn setup(&mut self, db: &Db, country_weights: &[(CountryId, u32)], total_weight: u32) {
        self.create_manager(db, country_weights, total_weight).await;
        self.return_actions_to_full(db).await;
        self.promote_junior_players(db, country_weights, total_weight).await;
    }

    // Give a few junior players to the team at the end of the season.
    async fn promote_junior_players(&mut self, db: &Db, country_weights: &[(CountryId, u32)], total_weight: u32) {
        for _ in 0..rand::random_range(1..=3) {
            let player = Player::build_and_save(db, country_weights, total_weight, 16, 19).await;
            Contract::build_from_years(db, player.person.id, self, 4, ContractRole::Player, true).await;
        }
    }

    pub async fn season_end_checker(&mut self, db: &Db, country_weights: &[(CountryId, u32)], total_weight: u32) {
        if self.is_season_end_date(db).await {
            self.promote_junior_players(db, country_weights, total_weight).await;
        }
    }
}