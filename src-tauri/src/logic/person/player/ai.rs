// Player-related AI.

use time::Date;

use crate::logic::{person::{attribute::PersonAttribute, contract::Contract, player::Player}, team::{Team, lineup::cache::LineUpCache}, types::Db};

impl Player {
    // Sign a given contract.
    pub async fn sign_contract(&self, db: &Db, today: Date, contract: Contract) {
        contract.sign(db, today).await;
        self.reject_contracts(db).await;
    }

    // Choose a contract to sign, if any.
    // This method assumes there are existing contract offers.
    pub async fn choose_contract(&self, db: &Db, today: Date, offers: &[Contract]) {
        let mut offer_evaluations = Vec::new();
        for offer in offers {
            offer_evaluations.push((self.evaluate_offer(db, &offer).await, offer));
        }

        offer_evaluations.sort_by(|a, b| b.0.total_cmp(&a.0));

        // If even the best offer is unacceptable, all should be rejected.
        if offer_evaluations[0].0 <= 0.0 {
            self.reject_contracts(db).await;
            return;
        }

        let contract = Self::get_best_offer(offer_evaluations);
        self.sign_contract(db, today, contract).await;
    }

    // Get the best contract.
    fn get_best_offer(offers: Vec<(f64, &Contract)>) -> Contract {
        let mut best_offers = Vec::new();

        let mut best_attraction = 0.0;
        for (attraction, offer) in offers.into_iter() {
            if best_offers.is_empty() {
                best_attraction = attraction;
                best_offers.push(offer);
            }
            else if best_attraction == attraction {
                best_offers.push(offer);
            }
        }

        let i = rand::random_range(0..best_offers.len());
        return best_offers.swap_remove(i).clone();
    }

    // Evaluate a contract offer.
    async fn evaluate_offer(&self, db: &Db, contract: &Contract) -> f64 {
        let mut team = Team::fetch_from_db(db, contract.team_id).await;
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
        let player_index = need.abilities.iter().position(|a| {
            *a == PersonAttribute::display(self.ability.value) as f64
        });
        if player_index.is_some() { need.abilities.remove(player_index.unwrap()); }

        // A player never wants to join a team where their playing time is uncertain.
        if PersonAttribute::display(self.ability.value) as f64 <= need.get_worst() {
            return -1000.0;
        }

        // 1.0 at best, above 0.0 at worst.
        let role_modifier = need.get_role_of_player(self);

        team.auto_build_lineup(db).await;
        let lineup = LineUpCache::build(db, &team.lineup).await;

        // Adding 10 so an empty roster is not that bad of a detriment.
        let avg_ability = 10.0 + lineup.average_ability();

        return avg_ability / role_modifier;
    }

    // The player checks if they are going to retire.
    pub async fn retires(&self, db: &Db, offers: &[Contract]) -> bool {
        // A player under contract or receiving offers will never retire.
        if self.person.contract(db).await.is_some() ||
        !offers.is_empty() {
            return false;
        }

        // 1 in 2000 chance to retire.
        return rand::random_bool(0.0005);
    }
}