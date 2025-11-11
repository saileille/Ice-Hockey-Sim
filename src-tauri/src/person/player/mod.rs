pub mod position;
mod ai;

use rand::rngs::ThreadRng;
use serde_json::json;
use time::Date;

use crate::{
    database::PLAYERS, person::{Gender, attribute::{AttributeId, PersonAttribute}}, time::date_to_db_string, types::{PlayerId, TeamId, convert}
};
use super::Person;
use self::position::{Position, PositionId};

#[derive(Debug)]
#[derive(Default, Clone)]
pub struct Player {
    pub id: PlayerId,
    pub person: Person,
    pub ability: PersonAttribute,
    pub position_id: PositionId,
}

// Basics.
impl Player {
    fn build(person: Person, position_id: PositionId) -> Self {
        Self {
            id: convert::int::<usize, PlayerId>(PLAYERS.lock().unwrap().len() + 1),
            person: person,
            ability: PersonAttribute::build(AttributeId::General, 0),
            position_id: position_id,
            ..Default::default()
        }
    }

    // Create a player and store it in the database. Return a clone of the Player.
    pub fn build_and_save(today: &Date, rng: &mut ThreadRng, min_age: u8, max_age: u8) -> Self {
        let player = Self::create(today, rng, min_age, max_age);
        player.save();
        return player;
    }

    // Just like build and save, but minimal arguments.
    pub fn create(today: &Date, rng: &mut ThreadRng, min_age: u8, max_age: u8) -> Self {
        let person = Person::create(today, rng, min_age, max_age, Gender::Male);
        let position_id = PositionId::get_random(rng);

        let mut player = Self::build(person, position_id);
        player.create_ability(today, rng);

        return player;
    }

    // Create the ability of a player during its generation.
    // Simulate the player's training for every day of their life so far.
    fn create_ability(&mut self, today: &Date, rng: &mut ThreadRng) {
        let days = self.person.get_age_days(today);
        for i in 0..days {
            self.train(rng, i);
        }
    }

    // Get a player from the database.
    pub fn fetch_from_db(id: &PlayerId) -> Option<Self> {
        PLAYERS.lock().unwrap().get(id).cloned()
    }

    // Update the Team to database.
    pub fn save(&self) {
        PLAYERS.lock().unwrap().insert(self.id, self.clone());
    }

    // Delete the Player from the database.
    pub fn delete_from_db(&self) {
        PLAYERS.lock().unwrap().remove(&self.id);
    }

    // Get a clone of the player's position.
    fn get_position(&self) -> Position {
        Position::fetch_from_db(&self.position_id)
    }

    // Get all free agents from the database with given positions, which the given team has not approached yet.
    pub fn get_free_agents_for_team(positions: Vec<&PositionId>, team_id: TeamId) -> Vec<Self> {
        PLAYERS.lock().unwrap().iter().filter_map(|(_, a)| {
            match a.person.contract.is_none() && a.person.is_active && positions.contains(&&a.position_id) {
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

    // Get all free agents in the game.
    pub fn get_all_free_agents_package(today: &Date) -> serde_json::Value {
        let mut players: Vec<Self> = PLAYERS.lock().unwrap().iter().filter_map(|(_, a)| {
            match a.person.contract.is_none() && a.person.is_active {
                true => Some(a.clone()),
                _ => None
            }
        }).collect();

        players.sort_by(|a, b|
            b.ability.get_display().cmp(&a.ability.get_display())
            .then((a.position_id.clone() as u8).cmp(&(b.position_id.clone() as u8)))
            .then(a.person.surname.cmp(&b.person.surname))
            .then(a.person.forename.cmp(&b.person.forename))
        );

        players.iter().map(|a| a.get_package(today)).collect()
    }

    // Get relevant information of the player.
    pub fn get_package(&self, today: &Date) -> serde_json::Value {
        let contract = match self.person.contract.as_ref() {
            Some(contract) => Some(contract.get_package(today)),
            _ => None
        };

        let contract_offers: Vec<serde_json::Value> = self.person.contract_offers.iter().map(|a| a.get_package(today)).collect();

        json!({
            "id": self.id,
            "name": self.person.get_full_name(),
            "country": self.person.get_country().name,
            "position": self.get_position().abbreviation,
            "age": self.person.get_age_years(today),
            "birthday": date_to_db_string(&self.person.birthday),
            "ability": self.ability.get_display(),
            "real_ability": self.ability.get(),
            "contract": contract,
            "offers": contract_offers
        })
    }

    // Get the position and the ID of the player.
    pub fn get_roster_overview_package(&self, in_roster: bool) -> serde_json::Value {
        json!({
            "position": Position::fetch_from_db(&self.position_id).abbreviation,
            "id": self.id,
            "in_roster": in_roster,
        })
    }

    // The daily training of the player.
    pub fn daily_training(&mut self, today: &Date, rng: &mut ThreadRng) {
        self.train(rng, self.person.get_age_days(today));
    }

    // Do the training (also used in player generation).
    fn train(&mut self, rng: &mut ThreadRng, age_days: u16) {
        self.ability.update(age_days, rng);
    }
}