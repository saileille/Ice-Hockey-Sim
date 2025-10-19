// Team AI stuff...

use std::collections::HashMap;

use time::Date;

use crate::{competition::Competition, person::{player::{position::{Position, PositionId}, Player}, Contract}, team::Team, time::date_to_db_string, types::{convert, PlayerId}};

#[derive(Debug)]
#[derive(Default, Clone)]
pub struct PlayerNeed {
    position: PositionId,

    // Abilities of the players.
    abilities: Vec<f64>,

    // Calculated and set in get_urgency
    // f64::MAX: Must have this type of player at all costs.
    // f64::MIN: Will not acquire a player of this type (unless maybe if one is *really* good).
    urgency: f64,
}

impl PlayerNeed {
    // Build the element.
    fn build(position: PositionId) -> Self {
        let mut need = Self::default();
        need.position = position;
        return need;
    }

    // Get how many players the team has that have to be left outside a match lineup.
    fn get_surplus(&self) -> i8 {
        convert::usize_to_i8(self.abilities.len()) - self.get_lineup_places()
    }

    // Get the average ability of the players.
    fn get_avg_ability(&self) -> f64 {
        let total: f64 = self.abilities.iter().sum();
        return total / convert::usize_to_f64(self.abilities.len());
    }

    // Get the worst ability of a player in lineup.
    fn get_worst(&self) -> f64 {
        match self.get_surplus() < 0 {
            true => 0.0,
            _ => *self.abilities.first().unwrap()
        }
    }


    // Get the best ability of a player in lineup.
    fn get_best(&self) -> f64 {
        match self.abilities.last() {
            Some(a) => *a,
            _ => 0.0
        }
    }

    // Calculate how much a team wants this type of player.
    fn calculate_urgency(&mut self, needs: &[Self]) {
        // If the team does not have enough players to play, something ought to be done about it...
        if self.get_surplus() < 0 {
            self.urgency = 10000.0; // Arbitrary
            return;
        }

        // Team will not hire more players if they have double of what they can fit in lineup.
        else if self.get_surplus() >= self.get_lineup_places() {
            self.urgency = -10000.0;    // Arbitrary.
            return;
        }

        let total_position_ability: f64 = needs.iter().map(|a| a.get_avg_ability()).sum();
        let avg_position_ability = total_position_ability / convert::usize_to_f64(needs.len());

        // The lower the quality in position, the bigger the modifier.
        // Lowest possible: 0.167, highest that is not arbitrary max: 1276.
        let ability_modifier = if self.get_avg_ability() == 0.0 {
            if avg_position_ability == 0.0 { 1.0 }
            else { 2000.0 /* Arbitrary */ }
        }
        else {
            avg_position_ability / self.get_avg_ability()
        };

        // Could have more stuff baked into it, but this should do for now.
        self.urgency = ability_modifier;
    }

    // Get how many players of this particular position are allowed in lineup.
    // Does not take into account the possible variable lineup sizes of different competitions.
    fn get_lineup_places(&self) -> i8 {
        match self.position {
            PositionId::Goalkeeper => 2,
            _ => 4
        }
    }

    // Evaluate the team's desire to acquire given player.
    // Do not bother if the value is negative.
    fn evaluate_player(&self, player: &Player) -> f64 {
        let worst = match self.get_surplus() {
            i8::MIN..0 => 0.0,
            _ => self.get_worst()
        };

        (player.ability as f64 - worst) * self.urgency
    }
}

impl Team {
    // Team evaluates what kind of players it might need, and how desperately.
    // This is a costly operation, so only do it after a signing, removal of a player target, or contract expiration.
    // Needs re-evaluation once player development becomes a thing.
    pub fn evaluate_player_needs(&mut self) {
        let mut roster_build = self.get_players();
        roster_build.append(&mut self.get_approached_players());
        roster_build.sort_by(|a, b| b.ability.cmp(&a.ability));
        let players = get_players_per_position(roster_build);


        self.player_needs = players.iter().map(|(k, v)| evaluate_position_needs(k, v)).collect();

        let needs_clone = self.player_needs.clone();
        for i in 0..self.player_needs.len() {
            let need = &mut self.player_needs[i];
            need.calculate_urgency(&needs_clone);
        }

        self.player_needs.sort_by(|a, b| b.urgency.total_cmp(&a.urgency));
    }

    // Offer contract to a player, if the team needs one.
    // Return whether contract was offered or not.
    pub fn offer_contract(&mut self, today: &Date) -> bool {
        let mut player = self.select_player_from_shortlist();
        if player.is_none() { return false; }
        self.offer_contract_to_player(player.as_mut().unwrap(), today);
        return true;
    }

    // Give the team an opportunity to offer a contract to a player.
    // Assumes that self.player_needs is up-to-date!
    fn select_player_from_shortlist(&self) -> Option<Player> {
        let free_agents = self.get_player_shortlist();

        // Do not offer any contracts if there is no-one the team wants.
        if free_agents.is_empty() { return None; }

        // How much a team wants a specific player.
        let mut player_attraction: Vec<(PlayerId, f64)> = Vec::new();
        let mut total_weight: f64 = 0.0;
        for player in free_agents.iter() {
            for need in self.player_needs.iter() {
                if need.position == player.position_id {
                    let evaluation = need.evaluate_player(player);

                    // Do not add these.
                    if evaluation <= 0.0 { continue; }

                    player_attraction.push((player.id, evaluation));

                    total_weight += evaluation;
                    break;
                }
            }
        }

        if player_attraction.is_empty() { return None; }

        // Weighted random to determine who the team is trying to sign.
        let value = rand::random_range(0.0..total_weight);
        let mut counter = 0.0;
        for (id, evaluation) in player_attraction.iter() {
            counter += evaluation;
            if value < counter {
                return Some(Player::fetch_from_db(id).unwrap());
            }
        }

        panic!("impossibru. value was {}, total weight was {}", value, total_weight);
    }

    // Offer contract to a given player.
    fn offer_contract_to_player(&mut self, player: &mut Player, today: &Date) {
        let years = rand::random_range(1..=4);  // 1-4 year contract offers, just like MHM.

        let comp = Competition::fetch_from_db(&self.primary_comp_id);
        let end_date = comp.season_window.end.get_previous_date_with_year_offset(years);

        let contract = Contract::build(&date_to_db_string(today), &date_to_db_string(&end_date), self.id);
        player.person.contract_offers.push(contract);
        self.approached_players.push(player.id);
        self.evaluate_player_needs();
        player.save();
    }

    // Get a player shortlist of possible hirelings.
    fn get_player_shortlist(&self) -> Vec<Player> {
        let mut positions = vec![&self.player_needs[0].position];
        let highest_urgency = self.player_needs[0].urgency;

        // No players returned if all urgencies are negative.
        if highest_urgency < 0.0 { return Vec::new(); }

        for need in &self.player_needs[1..self.player_needs.len()] {
            if need.urgency == highest_urgency || need.urgency >= 10000.0 {
                positions.push(&need.position);
            }
        }

        let mut free_agents = Player::get_free_agents_for_team(positions, self.id);
        free_agents.sort_by(|a, b| b.ability.cmp(&a.ability));

        return free_agents;
    }
}

// Get a HashMap of players and their positions.
fn get_players_per_position(players: Vec<Player>) -> HashMap<PositionId, Vec<Player>> {
    let mut players_by_position: HashMap<PositionId, Vec<Player>> = HashMap::from([
        (PositionId::Goalkeeper, Vec::new()),
        (PositionId::LeftDefender, Vec::new()),
        (PositionId::RightDefender, Vec::new()),
        (PositionId::LeftWinger, Vec::new()),
        (PositionId::Centre, Vec::new()),
        (PositionId::RightWinger, Vec::new()),
    ]);
    for player in players {
        players_by_position.get_mut(&player.position_id).unwrap().push(player);
    }

    return players_by_position;
}

// Evaluate the need for a specific position.
fn evaluate_position_needs(position: &PositionId, players: &[Player]) -> PlayerNeed {
    let mut need = PlayerNeed::build(position.clone());
    let players_in_lineup = players.len().clamp(0, need.get_lineup_places() as usize);
    need.abilities = players[0..players_in_lineup].iter().map(|a| a.ability as f64).collect();

    return need;
}