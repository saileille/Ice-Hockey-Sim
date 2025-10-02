pub mod position;

use super::{Person, Gender};
use self::position::{Position, PositionId};
use crate::database::PLAYERS;

#[derive(PartialEq, Default, Clone, Debug)]
pub struct Player {
    pub id: usize,  // id: 0 is reserved
    pub person: Person,
    pub ability: u8,
    pub position_id: PositionId,
}

impl Player {   // Basics.
    fn new(country_id: usize, ability: u8, position_id: PositionId) -> Self {
        let mut player: Self = Self::default();
        player.person = Person::new(country_id, Gender::Male);
        player.ability = ability;
        player.position_id = position_id;

        return player;
    }

    // Create a player and store it in the database. Return a clone of the Player.
    pub fn create_and_save(country_id: usize, ability: u8, position_id: PositionId) -> Self {
        let mut player: Self = Self::new(country_id, ability, position_id);
        player.id = PLAYERS.lock().unwrap().len();
        
        player.update_to_db();
        return player;
    }

    // Get a player from the database.
    pub fn fetch_from_db(id: &usize) -> Self {
        PLAYERS.lock().unwrap().get(id).expect(&format!("no Player with id {id}")).clone()
    }

    // Update the Team to database.
    pub fn update_to_db(&self) {
        PLAYERS.lock()
            .expect(&format!("something went wrong when trying to update Player {}: {} to PLAYERS", self.id, self.person.get_full_name())).insert(self.id, self.clone());
    }

    // Check if the player in question is not the default placeholder.
    pub fn is_valid(&self) -> bool {
        self.id != 0 &&
        self.person.is_valid() &&
        self.position_id != PositionId::default()
    }

    // Get a clone of the player's position.
    fn get_position_clone(&self) -> Position {
        Position::fetch_from_db(&self.position_id)
    }
}