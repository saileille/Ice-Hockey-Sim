pub mod lineup;

use std::collections::HashSet;
use rand::{
    distr::Uniform,
    Rng
};
use crate::{
    types::{
        CountryId,
        TeamId,
        PlayerId
    },
    database::TEAMS,
    country::Country,
    person::player::{
        Player,
        position::PositionId
    }
};
use self::lineup::LineUp;

#[derive(Default, Clone)]
pub struct Team {
    pub id: TeamId,  // id: 0 is reserved
    pub name: String,
    pub roster: HashSet<PlayerId>,
    pub lineup: LineUp,
}

impl Team { // Basics.
    // Create a new ID.
    fn create_id(&mut self, id: usize) {
        self.id = match id.try_into() {
            Ok(n) => n,
            Err(e) => panic!("{e}")
        };
    }

    fn build<S: AsRef<str>>(name: S) -> Self {
        let mut team: Team = Team::default();
        team.name = String::from(name.as_ref());

        return team;
    }

    // Create a team and store it in the database. Return a clone of the Team.
    pub fn build_and_save<S: AsRef<str>>(name: S) -> Self {
        let mut team: Self = Self::build(name);
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

    // Get every player in the roster as a clone.
    fn get_players(&self) -> Vec<Player> {
        let mut players: Vec<Player> = Vec::new();

        for id in self.roster.iter() {
            players.push(Player::fetch_from_db(id).unwrap());
        }

        return players;
    }
}

impl Team {
    // Build a lineup for the team from its roster.
    pub fn auto_build_lineup(&mut self) {
        self.lineup.clear();

        let mut players: Vec<Player> = self.get_players();
        players.sort_by(|a, b| b.ability.cmp(&a.ability));

        self.lineup.auto_add(players);
        self.save();
    }
}

// Tests.
impl Team {
    // Generate a basic roster of players for the team.
    fn generate_roster(&mut self, min_ability: u8, max_ability: u8) {
        self.roster = HashSet::new();
        let range: Uniform<u8> = Uniform::new_inclusive(min_ability, max_ability)
            .expect(&format!("error: low: {min_ability}, high: {max_ability}"));

        let mut rng: rand::prelude::ThreadRng = rand::rng();

        let country_id: CountryId = Country::fetch_from_db_with_name("Finland").id;

        // Goalkeepers...
        for _ in 0..2 {
            let player: Player = Player::build_and_save(country_id, rng.sample(range), PositionId::Goalkeeper);
            self.roster.insert(player.id);
        }

        // Defenders...
        for _ in 0..8 {
            let player: Player = Player::build_and_save(country_id, rng.sample(range), PositionId::Defender);
            self.roster.insert(player.id);
        }

        // Left Wingers...
        for _ in 0..4 {
            let player: Player = Player::build_and_save(country_id, rng.sample(range), PositionId::LeftWinger);
            self.roster.insert(player.id);
        }

        // Centres...
        for _ in 0..4 {
            let player: Player = Player::build_and_save(country_id, rng.sample(range), PositionId::Centre);
            self.roster.insert(player.id);
        }

        // Right Wingers...
        for _ in 0..4 {
            let player: Player = Player::build_and_save(country_id, rng.sample(range), PositionId::RightWinger);
            self.roster.insert(player.id);
        }

        self.save();
    }

    // Delete the team's players.
    pub fn delete_players(&mut self) {
        for id in self.roster.iter() {
            Player::fetch_from_db(id).unwrap().delete_from_db();
        }

        self.roster.clear();
    }

    // Setup a team before a season.
    pub fn setup(&mut self, min_ability: u8, max_ability: u8) {
        self.generate_roster(min_ability, max_ability);
    }
}