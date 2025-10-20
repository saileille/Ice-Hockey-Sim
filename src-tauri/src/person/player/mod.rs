pub mod position;
mod ai;

use rand::{rngs::ThreadRng, Rng};
use rand::seq::IteratorRandom;
use serde_json::json;

use crate::{
    database::{COUNTRIES, PLAYERS, POSITIONS}, types::{CountryId, PlayerId, TeamId}
};
use super::{Person, Gender};
use self::position::{Position, PositionId};

#[derive(Debug)]
#[derive(Default, Clone)]
pub struct Player {
    pub id: PlayerId,
    pub person: Person,
    pub ability: u8,
    pub position_id: PositionId,
}

// Basics.
impl Player {
    // Create a new ID.
    fn create_id(&mut self, id: usize) {
        self.id = match id.try_into() {
            Ok(n) => n,
            Err(e) => panic!("{e}")
        };
    }

    fn build(person: Person, ability: u8, position_id: PositionId) -> Self {
        let mut player = Self::default();
        player.person = person;
        player.ability = ability;
        player.position_id = position_id;

        return player;
    }

    // Create a player and store it in the database. Return a clone of the Player.
    pub fn build_and_save(person: Person, ability: u8, position_id: PositionId) -> Self {
        let mut player = Self::build(person, ability, position_id);
        player.create_id(PLAYERS.lock().unwrap().len() + 1);
        player.save();
        return player;
    }

    // Just like build and save, but no arguments.
    pub fn build_and_save_random(rng: &mut ThreadRng) -> Self {
        let person = Person::build_random();

        let ability = rng.random_range(0..=u8::MAX);
        let (position_id, _) = POSITIONS.iter().choose(rng).unwrap();

        return Self::build_and_save(person, ability, position_id.clone());
    }

    // Get a player from the database.
    pub fn fetch_from_db(id: &PlayerId) -> Option<Self> {
        PLAYERS.lock().unwrap().get(id).cloned()
    }

    // Update the Team to database.
    pub fn save(&self) {
        PLAYERS.lock()
            .expect(&format!("something went wrong when trying to update Player {}: {} to PLAYERS", self.id, self.person.get_full_name())).insert(self.id, self.clone());
    }

    // Delete the Player from the database.
    pub fn delete_from_db(&self) {
        PLAYERS.lock()
            .expect(&format!("something went wrong when trying to delete Player {}: {} from PLAYERS", self.id, self.person.get_full_name()))
            .remove(&self.id);
    }

    // Check if the player in question is not the default placeholder.
    pub fn is_valid(&self) -> bool {
        self.id != 0 &&
        self.person.is_valid() &&
        self.position_id != PositionId::default()
    }

    // Get a clone of the player's position.
    fn get_position(&self) -> Position {
        Position::fetch_from_db(&self.position_id)
    }

    // Get all free agents from the database with given positions, which the given team has not approached yet.
    pub fn get_free_agents_for_team(positions: Vec<&PositionId>, team_id: TeamId) -> Vec<Self> {
        PLAYERS.lock().unwrap().iter().filter_map(|(_, a)| {
            match a.person.contract.is_none() && positions.contains(&&a.position_id) {
                true => {
                    let mut team_has_offer = false;
                    for offer in a.person.contract_offers.iter() {
                        if offer.team_id == team_id {
                            team_has_offer = true;
                            break;
                        }
                    }

                    match team_has_offer {
                        true => None,
                        _ => Some(a.clone())
                    }
                },

                _ => None
            }
        }).collect()
    }

    // Get relevant information of the player for team screen.
    pub fn get_team_screen_json(&self) -> serde_json::Value {
        let seasons_left = match self.person.contract.as_ref() {
            Some(contract) => contract.get_seasons_left(),
            _ => 0
        };

        json!({
            "id": self.id,
            "name": self.person.get_full_name(),
            "country": self.person.get_country().name,
            "position": self.get_position().abbreviation,
            "ability": self.ability,
            "seasons_left": seasons_left,
        })
    }

    // Get relevant information of the player for the player screen.
    pub fn get_player_screen_json(&self) -> serde_json::Value {
        let contract = match self.person.contract.as_ref() {
            Some(contract) => Some(contract.get_person_screen_json()),
            _ => None
        };

        let contract_offers: Vec<serde_json::Value> = self.person.contract_offers.iter().map(|a| a.get_person_screen_json()).collect();

        json!({
            "name": self.person.get_full_name(),
            "country": self.person.get_country().name,
            "position": self.get_position().abbreviation,
            "ability": self.ability,
            "contract": contract,
            "offers": contract_offers
        })
    }
}