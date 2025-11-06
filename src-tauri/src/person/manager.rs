// This is what a player is!

use serde_json::json;

use crate::{database::MANAGERS, person::{Gender, Person}, types::{CountryId, ManagerId}};

#[derive(Default, Clone)]
pub struct Manager {
    pub id: ManagerId,
    pub person: Person,
    pub is_human: bool,
}

impl Manager {
    // Create a new ID.
    fn create_id(&mut self, id: usize) {
        self.id = match id.try_into() {
            Ok(n) => n,
            Err(e) => panic!("{e}")
        };
    }

    // Build a manager.
    fn build(person: Person) -> Self {
        let mut manager = Self::default();
        manager.person = person;

        return manager;
    }

    // Create a manager and store it in the database. Return a clone of the Manager.
    fn build_and_save(person: Person) -> Self {
        let mut player = Self::build(person);
        player.create_id(MANAGERS.lock().unwrap().len() + 1);
        player.save();
        return player;
    }

    // Build a random manager.
    pub fn build_and_save_random() -> Self {
        let person = Person::build_random();
        return Self::build_and_save(person);
    }

    // Get a manager from the database.
    pub fn fetch_from_db(id: &ManagerId) -> Option<Self> {
        MANAGERS.lock().unwrap().get(id).cloned()
    }

    // Update the manager to database.
    pub fn save(&self) {
        MANAGERS.lock().unwrap().insert(self.id, self.clone());
    }

    // Delete the manager from the database.
    pub fn delete_from_db(&self) {
        MANAGERS.lock().unwrap().remove(&self.id);
    }

    // Get the human manager.
    pub fn get_human() -> Option<Self> {
        for manager in MANAGERS.lock().unwrap().values() {
            if manager.is_human { return Some(manager.clone()); }
        }

        return None;
    }

    // Get relevant information to the team screen.
    pub fn get_team_screen_json(&self) -> serde_json::Value {
        json!({
            "name": self.person.get_full_name()
        })
    }

    // Get a package of information that is used in game interaction.
    // For human players only.
    pub fn get_package(&self) -> serde_json::Value {
        json!({
            "team": match self.person.contract.as_ref() {
                Some(contract) => Some(contract.get_team().get_manager_package()),
                _ => None
            }
        })
    }
}