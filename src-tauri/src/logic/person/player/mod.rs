pub mod position;
mod ai;

use rand::rngs::ThreadRng;
use serde_json::json;
use time::Date;

use crate::logic::{app_data::AppData, person::{Gender, Person, attribute::{AttributeId, PersonAttribute}, player::position::PositionId}, types::Db};

#[derive(Debug)]
#[derive(Default, Clone)]
pub struct Player {
    pub person: Person,
    pub ability: PersonAttribute,
    pub position_id: PositionId,
}

// Basics.
impl Player {
    fn build(person: Person, position_id: PositionId) -> Self {
        Self {
            person: person,
            ability: PersonAttribute::build(AttributeId::General, 0),
            position_id: position_id,
            ..Default::default()
        }
    }

    // Create a player and store it in the database. Return a clone of the Player.
    pub async fn build_and_save(data: &AppData, today: Date, min_age: u8, max_age: u8) -> Self {
        let player = Self::create(data, today, min_age, max_age).await;
        player.save(&data.db).await;
        return player;
    }

    // Just like build and save, but minimal arguments.
    pub async fn create(data: &AppData, today: Date, min_age: u8, max_age: u8) -> Self {
        let person = Person::create(data, today, min_age, max_age, Gender::Male).await;
        let position_id = PositionId::get_random();

        let mut player = Self::build(person, position_id);
        player.create_ability(&data.db, today).await;

        return player;
    }

    // Create the ability of a player during its generation.
    // Simulate the player's training for every day of their life so far.
    async fn create_ability(&mut self, db: &Db, today: Date) {
        let days = self.person.age_in_days(today);
        let mut rng = rand::rng();
        for i in 0..days {
            self.train(&mut rng, i);
        }
    }

    // Get all free agents in the game.
    pub async fn free_agents_package(db: &Db, today: Date) -> Vec<serde_json::Value> {
        let mut players = Self::free_agents(db).await;

        players.sort_by(|a, b| {
            PersonAttribute::display(b.ability.value)
            .cmp(&PersonAttribute::display(a.ability.value))
        });

        let mut json = Vec::new();
        for player in players {
            json.push(player.package(db, today).await);
        }
        return json;
    }

    // Get relevant information of the player.
    pub async fn package(&self, db: &Db, today: Date) -> serde_json::Value {
        json!({
            "person": self.person.package(db, today).await,
            "id": self.person.id,
            "position": self.position_abbr(db).await,
            "ability": PersonAttribute::display(self.ability.value),
        })
    }

    // Get the position and the ID of the player.
    pub async fn roster_overview_package(&self, db: &Db, in_roster: bool) -> serde_json::Value {
        json!({
            "position": self.position_abbr(db).await,
            "id": self.person.id,
            "in_roster": in_roster,
        })
    }

    // The daily training of the player.
    pub async fn daily_training(&mut self, db: &Db, today: Date) {
        self.train(&mut rand::rng(), self.person.age_in_days(today));
        self.update_ability(db).await;
    }

    // Do the training (also used in player generation).
    fn train(&mut self, rng: &mut ThreadRng, age_in_days: u16) {
        self.ability.update(rng, age_in_days);
    }
}