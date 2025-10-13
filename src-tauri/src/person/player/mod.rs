pub mod position;

use crate::{
    types::{CountryId, PlayerId},
    database::PLAYERS
};
use super::{Person, Gender};
use self::position::{Position, PositionId};

#[derive(Default, Clone)]
pub struct Player {
    pub id: PlayerId,  // id: 0 is reserved
    pub person: Person,
    pub ability: u8,
    pub position_id: PositionId,
}

impl Player {   // Basics.
    // Create a new ID.
    fn create_id(&mut self, id: usize) {
        self.id = match id.try_into() {
            Ok(n) => n,
            Err(e) => panic!("{e}")
        };
    }

    fn build(country_id: CountryId, ability: u8, position_id: PositionId) -> Self {
        let mut player = Self::default();
        player.person = Person::build(country_id, Gender::Male);
        player.ability = ability;
        player.position_id = position_id;

        return player;
    }

    // Create a player and store it in the database. Return a clone of the Player.
    pub fn build_and_save(country_id: CountryId, ability: u8, position_id: PositionId) -> Self {
        let mut player = Self::build(country_id, ability, position_id);
        player.create_id(PLAYERS.lock().unwrap().len() + 1);
        player.save();
        return player;
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
}