// This is what a player is!

use std::collections::HashMap;

use rand::rngs::ThreadRng;
use serde_json::json;
use time::Date;

use crate::{country::Country, database::MANAGERS, person::{Gender, Person}, types::{CountryId, ManagerId, convert}};

#[derive(Default, Clone)]
pub struct Manager {
    pub id: ManagerId,
    pub person: Person,
    pub is_human: bool,
}

impl Manager {
    // Build a manager.
    fn build(person: Person) -> Self {
        Self {
            id: convert::int::<usize, ManagerId>(MANAGERS.lock().unwrap().len() + 1),
            person: person,
            ..Default::default()
        }
    }

    // Create a manager and store it in the database. Return a clone of the Manager.
    fn build_and_save(person: Person) -> Self {
        let manager = Self::build(person);
        manager.save();
        return manager;
    }

    // Build a random manager.
    pub fn build_and_save_random(countries: &HashMap<CountryId, Country>, today: &Date, rng: &mut ThreadRng) -> Self {
        let person = Person::create(countries, today, rng, 30, 60, Gender::Male);
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
    pub fn get_team_screen_package(&self, today: &Date) -> serde_json::Value {
        json!({
            "person": self.person.get_package(today)
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