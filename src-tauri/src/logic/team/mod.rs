pub mod lineup;
pub mod ai;

use serde_json::json;
use sqlx::FromRow;
use time::Date;

use crate::logic::{app_data::AppData, person::{attribute::PersonAttribute, contract::{Contract, ContractRole}, manager::Manager, player::Player}, team::{ai::PlayerNeed, lineup::LineUp}, types::{CompetitionId, Db, PersonId, TeamId}};

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

impl Team {
    pub fn build(name: &str) -> Self {
        Self {
            full_name: name.to_string(),
            ..Default::default()
        }
    }

    // Get info for a team screen in JSON.
    pub async fn team_screen_package(&self, db: &Db, today: Date) -> serde_json::Value {
        let mut players = self.players_and_approached(db).await;

        players.sort_by(|a, b| {
            (a.position_id.clone() as u8).cmp(&(b.position_id.clone() as u8))

            .then(PersonAttribute::display(b.ability.value)
            .cmp(&PersonAttribute::display(a.ability.value)))
        });

        let mut json_players = Vec::new();
        for player in players {
            json_players.push(player.package(db, today).await);
        }

        json!({
            "id": self.id,
            "name": self.full_name,
            "manager": match self.manager(db).await {
                Some(manager) => Some(manager.team_screen_package(db, today).await),
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

    // Build a lineup for the team from its roster and save to database.
    pub async fn auto_build_lineup(&mut self, db: &Db) {
        self.lineup.clear();

        let mut players = self.players(db).await;
        players.sort_by(|a, b| {
            PersonAttribute::display(b.ability.value)
            .cmp(&PersonAttribute::display(a.ability.value))
        });

        self.lineup.auto_add(players);
        self.lineup.save(self.id, db).await;
    }

    async fn remove_action(&mut self, db: &Db) {
        self.set_actions_remaining(db, self.actions_remaining - 1).await;
    }

    // Give the team its full actions back.
    // Action value could depend on quantity and quality of team staff?
    pub async fn return_actions_to_full(&mut self, db: &Db) {
        self.set_actions_remaining(db, 1).await;
    }

    // Create a manager out of thin air.
    async fn create_manager(&mut self, data: &AppData, today: Date) {
        let manager = Manager::build_and_save_random(data, today, false).await;
        Contract::build_from_years(&data.db, today, manager.person.id, self, 100, ContractRole::Manager, true).await;
    }

    // Set up the team when initialising a game.
    pub async fn setup(&mut self, data: &AppData, today: Date) {
        self.create_manager(data, today).await;
        self.return_actions_to_full(&data.db).await;
        self.promote_junior_players(data, today).await;
    }

    // Give a few junior players to the team at the end of the season.
    async fn promote_junior_players(&mut self, data: &AppData, today: Date) {
        for _ in 0..rand::random_range(1..=3) {
            let player = Player::build_and_save(data, today, 16, 19).await;
            Contract::build_from_years(&data.db, today, player.person.id, self, 4, ContractRole::Player, true).await;
        }
    }

    pub async fn season_end_checker(&mut self, data: &AppData, today: Date) {
        if self.is_season_end_date(&data.db, today).await {
            self.promote_junior_players(data, today).await;
        }
    }
}