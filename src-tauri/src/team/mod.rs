pub mod lineup;
mod ai;

use std::{collections::HashSet, mem::discriminant};
use rand::{
    distr::Uniform,
    Rng
};
use serde_json::json;
use time::Date;
use crate::{
    country::Country, database::{TEAMS, TODAY}, person::{manager::Manager, player::{
        position::PositionId, Player
    }, Contract, Person}, team::ai::PlayerNeed, time::date_to_db_string, types::{
        CompetitionId, ManagerId, PlayerId, TeamId
    }
};
use self::lineup::LineUp;

#[derive(Default, Clone)]
pub struct Team {
    pub id: TeamId,
    pub name: String,
    pub roster: Vec<PlayerId>,
    pub manager_id: ManagerId,
    pub lineup: LineUp,
    pub primary_comp_id: CompetitionId,

    // Player-acquisition related.
    pub approached_players: Vec<PlayerId>,
    pub player_needs: Vec<PlayerNeed>,

    // Actions.
    pub actions_remaining: u8
}

// Basics.
impl Team {
    // Create a new ID.
    fn create_id(&mut self, id: usize) {
        self.id = match id.try_into() {
            Ok(n) => n,
            Err(e) => panic!("{e}")
        };
    }

    fn build(name: &str) -> Self {
        let mut team = Team::default();
        team.name = name.to_string();

        return team;
    }

    // Create a team and store it in the database. Return a clone of the Team.
    pub fn build_and_save(name: &str) -> Self {
        let mut team = Self::build(name);
        team.create_id(TEAMS.lock().unwrap().len() + 1);
        team.save();
        return team;
    }

    pub fn fetch_from_db(id: &TeamId) -> Self {
        TEAMS.lock().unwrap().get(id)
            .expect(&format!("no Team with id {id:#?}")).clone()
    }

    // Update the Team to database.
    pub fn save(&self) {
        TEAMS.lock()
            .expect(&format!("something went wrong when trying to update Team {}: {} to TEAMS", self.id, self.name))
            .insert(self.id, self.clone());
    }

    // Delete the Team from the database.
    pub fn delete_from_db(&self) {
        TEAMS.lock()
            .expect(&format!("something went wrong when trying to delete Team {}: {} from TEAMS", self.id, self.name))
            .remove(&self.id);
    }

    // Check that the team does not have illegal values.
    fn is_valid(&self) -> bool {
        self.id != 0 &&
        self.name != String::default() &&
        !self.roster.contains(&0)
    }

    // Get the team's manager.
    pub fn get_manager(&self) -> Option<Manager> {
        Manager::fetch_from_db(&self.manager_id)
    }

    // Get every player in the roster.
    fn get_players(&self) -> Vec<Player> {
        self.roster.iter().map(|id| Player::fetch_from_db(id).unwrap()).collect()
    }

    // Get the players to whom the team has offered contracts.
    fn get_approached_players(&self) -> Vec<Player> {
        self.approached_players.iter().map(|id| Player::fetch_from_db(id).unwrap()).collect()
    }

    // Get info for a team screen in JSON.
    pub fn get_team_screen_json(&self) -> serde_json::Value {
        let mut players = self.get_players();
        let mut approached_players = self.get_approached_players();
        players.append(&mut approached_players);

        players.sort_by(|a, b| (a.position_id.clone() as u8).cmp(&(b.position_id.clone() as u8)).then(b.ability.cmp(&a.ability)));

        let json_players: Vec<serde_json::Value> = players.iter().map(|a| a.get_team_screen_json()).collect();
        json!({
            "id": self.id,
            "name": self.name,
            "manager": match self.get_manager() {
                Some(manager) => Some(manager.get_team_screen_json()),
                _ => None
            },
            "players": json_players
        })
    }

    // Get relevant info for a player screen.
    pub fn get_player_screen_json(&self) -> serde_json::Value {
        json!({
            "id": self.id,
            "name": self.name
        })
    }

    // Get relevant info of the team for a human manager.
    pub fn get_manager_package_info(&self) -> serde_json::Value {
        json!({
            "id": self.id,
            "actions_remaining": self.actions_remaining,
            "approached_players": self.approached_players
        })
    }
}

impl Team {
    // Build a lineup for the team from its roster.
    pub fn auto_build_lineup(&mut self) {
        self.lineup.clear();

        let mut players = self.get_players();
        players.sort_by(|a, b| b.ability.cmp(&a.ability));

        self.lineup.auto_add(players);
        self.save();
    }

    // Give the team its full actions back.
    // Action value could depend on quantity and quality of team staff?
    pub fn return_actions_to_full(&mut self) {
        self.actions_remaining = 1;
    }
}

// Tests.
impl Team {
    // Generate a basic roster of players for the team.
    fn generate_roster(&mut self, min_ability: u8, max_ability: u8) {
        self.roster = Vec::new();
        let range = Uniform::new_inclusive(min_ability, max_ability)
            .expect(&format!("error: low: {min_ability}, high: {max_ability}"));

        let mut rng = rand::rng();
        // Goalkeepers...
        for _ in 0..2 {
            let player = Player::build_and_save(Person::build_random(), rng.sample(range), PositionId::Goalkeeper);
            self.roster.push(player.id);
        }

        // Left Defenders...
        for _ in 0..4 {
            let player = Player::build_and_save(Person::build_random(), rng.sample(range), PositionId::LeftDefender);
            self.roster.push(player.id);
        }

        // Right Defenders...
        for _ in 0..4 {
            let player = Player::build_and_save(Person::build_random(), rng.sample(range), PositionId::RightDefender);
            self.roster.push(player.id);
        }

        // Left Wingers...
        for _ in 0..4 {
            let player = Player::build_and_save(Person::build_random(), rng.sample(range), PositionId::LeftWinger);
            self.roster.push(player.id);
        }

        // Centres...
        for _ in 0..4 {
            let player = Player::build_and_save(Person::build_random(), rng.sample(range), PositionId::Centre);
            self.roster.push(player.id);
        }

        // Right Wingers...
        for _ in 0..4 {
            let player = Player::build_and_save(Person::build_random(), rng.sample(range), PositionId::RightWinger);
            self.roster.push(player.id);
        }
    }

    // Delete the team's players.
    fn delete_players(&mut self) {
        for id in self.roster.iter() {
            Player::fetch_from_db(id).unwrap().delete_from_db();
        }

        self.roster.clear();
    }

    // Create a manager out of thin air.
    fn create_manager(&mut self) {
        let mut manager = Manager::build_and_save_random();
        self.manager_id = manager.id;
        manager.person.contract = Some(Contract::build(&date_to_db_string(&TODAY.lock().unwrap()), &date_to_db_string(&Date::MAX), self.id));
        manager.save();
    }

    // Set up the team when initialising a game.
    pub fn setup(&mut self, min_ability: u8, max_ability: u8) {
        self.create_manager();
        // self.generate_roster(min_ability, max_ability);
        self.return_actions_to_full();
        self.save();
    }
}