// Player-related AI.

use time::Date;

use crate::{person::player::{position::Position, Player}, team::Team, time::date_to_db_string};

impl Player {
    // Choose a contract to sign from the available options.
    pub fn sign_contract(&mut self, today: &Date) {
        let mut contract = self.person.contract_offers.swap_remove(rand::random_range(0..self.person.contract_offers.len()));
        contract.start_date = date_to_db_string(today);

        self.person.contract = Some(contract);

        let mut team = Team::fetch_from_db(&self.person.contract.as_ref().unwrap().team_id);
        team.roster.push(self.id);
        team.approached_players.retain(|id| *id != self.id);
        team.save();

        for offer in self.person.contract_offers.iter() {
            let mut team = Team::fetch_from_db(&offer.team_id);
            team.approached_players.retain(|id| *id != self.id);
            team.evaluate_player_needs();
            team.save();
        }

        self.person.contract_offers.clear();
        println!("{} ({}, {}) accepted a contract offer from {}!", self.person.get_full_name(), Position::fetch_from_db(&self.position_id).abbreviation, self.ability, team.name);
    }
}