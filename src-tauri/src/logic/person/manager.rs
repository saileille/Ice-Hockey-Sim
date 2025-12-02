// This is what the player (user) is!
use serde_json::json;
use time::Date;

use crate::logic::{app_data::AppData, person::{Gender, Person}, types::Db};

// This struct is used when handling daily simulation.
#[derive(Default, Clone)]
pub struct Manager {
    pub person: Person,
    pub is_human: bool,
}

impl Manager {
    // Build a manager.
    fn build(person: Person, is_human: bool) -> Self {
        Self {
            person,
            is_human,
        }
    }

    // Create a manager and store it in the database. Return a clone of the Manager.
    async fn build_and_save(db: &Db, person: Person, is_human: bool) -> Self {
        let manager = Self::build(person, is_human);
        manager.save(db).await;
        return manager;
    }

    // Build a random manager.
    pub async fn build_and_save_random(data: &AppData, today: Date, is_human: bool) -> Self {
        let person = Person::create(data, today, 30, 60, Gender::Male).await;
        return Self::build_and_save(&data.db, person, is_human).await;
    }

    // Get relevant information to the team screen.
    pub async fn team_screen_package(&self, db: &Db, today: Date) -> serde_json::Value {
        json!({
            "person": self.person.package(db, today).await
        })
    }

    // Get a package of information that is used in game interaction.
    // For human players only.
    pub async fn package(&self, db: &Db) -> serde_json::Value {
        json!({
            "team": match self.person.contract(db).await {
                Some(contract) => Some(contract.team(db).await.manager_package(db).await),
                _ => None
            }
        })
    }
}