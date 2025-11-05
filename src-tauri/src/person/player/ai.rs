// Player-related AI.

use time::Date;

use crate::{person::{player::Player, Contract}, team::{lineup::cache::LineUpCache, Team}, time::date_to_db_string};

impl Player {
    // Sign a given contract.
    pub fn sign_contract(&mut self, mut contract: Contract, today: &Date) {
        contract.start_date = date_to_db_string(today);

        self.person.contract = Some(contract);

        let mut team = Team::fetch_from_db(&self.person.contract.as_ref().unwrap().team_id);
        team.roster.push(self.id);
        team.approached_players.retain(|id| *id != self.id);
        team.save();

        for offer in self.person.contract_offers.iter() {
            self.reject_contract(offer);
        }

        self.person.contract_offers.clear();
    }

    // Choose a contract to sign, if any.
    // This method assumes there are existing contract offers.
    pub fn choose_contract(&mut self, today: &Date) {
        let mut offers: Vec<(f64, &Contract)> = self.person.contract_offers.iter().map(|a| (self.evaluate_offer(a), a)).collect();
        offers.sort_by(|a, b| b.0.total_cmp(&a.0));

        // If even the best offer is unacceptable, all should be rejected.
        if offers[0].0 <= 0.0 {
            let mut rejected = Vec::new();
            for offer in self.person.contract_offers.iter() {
                self.reject_contract(offer);
                rejected.push(offer.get_team());
            }
            self.person.contract_offers.clear();
            return;
        }

        let team_id = offers[0].1.team_id;
        let index = self.person.contract_offers.iter().position(|a| a.team_id == team_id).unwrap();

        let contract = self.person.contract_offers.swap_remove(index);
        self.sign_contract(contract, today);
    }

    // Evaluate a contract offer.
    fn evaluate_offer(&self, contract: &Contract) -> f64 {
        let mut team = Team::fetch_from_db(&contract.team_id);
        let mut need_option = None;
        for team_need in team.player_needs.iter() {
            if team_need.position == self.position_id {
                need_option = Some(team_need.clone());
                break;
            }
        }

        // This should not be possible, but let's check against it just in case.
        if need_option.is_none() { return -1000.0; }

        // Let's unwrap so code becomes nicer.
        let mut need = need_option.unwrap();

        // Removing the player's ability from needs so the player does not compare against himself.
        let player_index = need.abilities.iter().position(|a| *a == self.ability.get_display() as f64);
        if player_index.is_some() { need.abilities.remove(player_index.unwrap()); }

        // A player never wants to join a team where their playing time is uncertain.
        if self.ability.get_display() as f64 <= need.get_worst() {
            return -1000.0;
        }

        // 1.0 at best, above 0.0 at worst.
        let role_modifier = need.get_role_of_player(self);

        team.auto_build_lineup();
        let lineup = LineUpCache::build(&team.lineup);

        // Adding 10 so an empty roster is not that bad of a detriment.
        let avg_ability = 10.0 + lineup.get_average_ability();

        return avg_ability / role_modifier;
    }

    // Reject a contract.
    fn reject_contract(&self, offer: &Contract) {
        let mut team = Team::fetch_from_db(&offer.team_id);
        team.approached_players.retain(|id| *id != self.id);
        team.save();
    }

    // Check if any contract offers for the player have expired.
    // Return if there are changes to the player.
    pub fn check_expired_offers(&mut self) -> bool {
        let mut expired = false;
        let mut expired_indexes = Vec::new();
        for (i, offer) in self.person.contract_offers.iter().enumerate() {
            if offer.check_if_expired() {
                self.reject_contract(offer);
                expired_indexes.push(i);
            }
        }

        if !expired_indexes.is_empty() {
            expired = true;
            for index in expired_indexes.iter().rev() {
                self.person.contract_offers.remove(*index);
            }
        }

        return expired;
    }
}