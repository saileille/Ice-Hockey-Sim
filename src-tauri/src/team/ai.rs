// Team AI stuff...

use std::collections::HashMap;

use rand::seq::IndexedRandom;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{person::{Contract, ContractRole, player::{Player, position::PositionId}}, team::Team, types::{Db, PersonId, convert}};

#[derive(Debug)]
#[derive(Default, Clone)]
#[derive(Serialize, Deserialize)]
pub struct PlayerNeed {
    pub position: PositionId,

    // Abilities of the players, from highest to lowest.
    pub abilities: Vec<f64>,

    // Calculated and set in urgency()
    // f64::MAX: Must have this type of player at all costs.
    // Negative: Will not acquire a player of this type (unless maybe if one is *really* good).
    urgency: f64,
}

impl PlayerNeed {
    // Build the element.
    fn build(position: PositionId) -> Self {
        Self {
            position: position,
            ..Default::default()
        }
    }

    // Get how many players the team has that have to be left outside a match lineup.
    fn surplus(&self) -> i8 {
        convert::int::<usize, i8>(self.abilities.len()) - self.lineup_places()
    }

    // Get the average ability of the players.
    fn avg_ability(&self) -> f64 {
        if self.abilities.len() == 0 { return 0.0 }

        let total: f64 = self.abilities.iter().sum();
        return total / convert::usize_to_f64(self.abilities.len());
    }

    // Get the worst ability of a player in lineup.
    pub fn get_worst(&self) -> f64 {
        match self.surplus() < 0 {
            true => 0.0,
            _ => *self.abilities.last().unwrap()
        }
    }

    // Get the best ability of a player in lineup.
    fn _get_best(&self) -> f64 {
        match self.abilities.first() {
            Some(a) => *a,
            _ => 0.0
        }
    }

    // See how attractive playing in the team would be to a player, role-wise.
    // 1.0 is the best it can be. This method assumes the player is getting a playable position,
    // so the minimum value is always above 0.0.
    pub fn get_role_of_player(&self, player: &Player) -> f64 {
        let mut index = 0.0;

        for ability in self.abilities.iter() {
            if player.ability.display() as f64 > *ability {
                break;
            }
            index += 1.0;
        }

        return 1.0 - (index / (self.lineup_places()) as f64);
    }

    // Calculate how much a team wants this type of player.
    fn calculate_urgency(&mut self, needs: &[Self]) {
        // If the team does not have enough players to play, something ought to be done about it...
        if self.surplus() < 0 {
            self.urgency = 10000.0; // Arbitrary
            return;
        }

        // Team will not hire more players if they have double of what they can fit in lineup.
        else if self.surplus() >= self.lineup_places() {
            self.urgency = -10000.0;    // Arbitrary.
            return;
        }

        let total_position_ability: f64 = needs.iter().map(|need| need.avg_ability()).sum();
        let avg_position_ability = total_position_ability / convert::usize_to_f64(needs.len());

        // The lower the quality in position, the bigger the modifier.
        // Lowest possible: 0.167, highest that is not arbitrary max: 1276.
        let ability_modifier = if self.avg_ability() == 0.0 {
            if avg_position_ability == 0.0 { 1.0 }
            else { 2000.0 /* Arbitrary */ }
        }
        else {
            avg_position_ability / self.avg_ability()
        };

        if ability_modifier.is_nan() {
            panic!(
                "ability modifier is nan\n\ntotal_position_ability: {}\navg_position_ability: {}\nself.avg_ability(): {}",
                total_position_ability, avg_position_ability, self.avg_ability()
            )
        }

        // Could have more stuff baked into it, but this should do for now.
        self.urgency = ability_modifier;
    }

    // Get how many players of this particular position are allowed in lineup.
    // Does not take into account the possible variable lineup sizes of different competitions.
    fn lineup_places(&self) -> i8 {
        match self.position {
            PositionId::Goalkeeper => 2,
            _ => 4
        }
    }

    // Evaluate the team's desire to acquire given player.
    // Do not bother if the value is negative.
    async fn evaluate_player(&self, db: &Db, player: &Player) -> f64 {
        let worst = match self.surplus() {
            i8::MIN..0 => 0.0,
            _ => self.get_worst()
        };

        (player.ability.display() as f64 - worst) * self.urgency * (1.0 / (player.person.no_of_offers(db).await + 1) as f64)
    }
}

impl Team {
    // Team evaluates what kind of players it might need, and how desperately.
    // Needs rework once player development becomes a thing.
    pub async fn evaluate_player_needs(&mut self, db: &Db) {
        let mut roster_build = self.players_and_approached(db).await;
        roster_build.sort_by(|a, b| b.ability.display().cmp(&a.ability.display()));

        let players = get_players_per_position(roster_build);
        self.player_needs = players.into_iter().map(|(k, v)| evaluate_position_needs(k, v)).collect();

        let needs_clone = self.player_needs.clone();
        for i in 0..self.player_needs.len() {
            let need = &mut self.player_needs[i];
            need.calculate_urgency(&needs_clone);
        }

        self.player_needs.sort_by(|a, b| b.urgency.total_cmp(&a.urgency));
        self.save_player_needs(db).await;
    }

    // Offer contract to a player, if the team needs one.
    // Return whether contract was offered or not.
    pub async fn offer_contract(&mut self, db: &Db) -> bool {
        let o_player = self.select_player_from_shortlist(db).await;
        if o_player.is_none() { return false; }
        let player = o_player.unwrap();

        self.create_contract_offer(db, player).await;
        return true;
    }

    // Give the team an opportunity to offer a contract to a player.
    // Assumes that self.player_needs is up-to-date!
    async fn select_player_from_shortlist(&self, db: &Db) -> Option<Player> {
        let free_agents = self.player_shortlist(db).await;

        // Do not offer any contracts if there is no-one the team wants.
        if free_agents.is_empty() { return None; }

        // How much a team wants a specific player.
        let mut player_attraction: Vec<(Player, f64)> = Vec::new();
        for player in free_agents {
            for need in self.player_needs.iter() {
                if need.position == player.position_id {
                    let evaluation = need.evaluate_player(db, &player).await;

                    // Do not add these.
                    if evaluation <= 0.0 { continue; }

                    player_attraction.push((player, evaluation));
                    break;
                }
            }
        }

        if player_attraction.is_empty() { return None; }
        player_attraction.sort_by(|(_, a), (_, b)| b.total_cmp(&a));

        // Choose randomly from equally good options.
        let mut choices: Vec<(Player, f64)> = Vec::new();
        for attraction in player_attraction {
            if choices.is_empty() || choices[0].1 <= attraction.1 {
                choices.push(attraction);
            }
        }

        let choice = choices.choose(&mut rand::rng()).unwrap();
        return Some(choice.0.clone());
    }

    // Create a contract offer from this team to a given person.
    pub async fn send_contract_offer(&mut self, db: &Db, person_id: PersonId, years: i32, role: ContractRole) {
        Contract::build_from_years(db, person_id, self, years, role, false).await;
        self.remove_action(db).await;
    }

    // AI makes a contract offer to a player.
    // The player itself should affect the contract offer at some point.
    pub async fn create_contract_offer(&mut self, db: &Db, player: Player) {
        let years = rand::random_range(1..=4);  // 1-4 year contract offers, just like MHM.
        self.send_contract_offer(db, player.person.id, years, ContractRole::Player).await;
    }

    // Get a player shortlist of possible hirelings.
    async fn player_shortlist(&self, db: &Db) -> Vec<Player> {
        let mut positions = vec![self.player_needs[0].position];
        let highest_urgency = self.player_needs[0].urgency;

        // No players returned if all urgencies are negative.
        if highest_urgency < 0.0 { return Vec::new(); }

        for need in &self.player_needs[1..self.player_needs.len()] {
            if need.urgency == highest_urgency || need.urgency >= 10000.0 {
                positions.push(need.position);
            }
        }

        let mut free_agents = Player::free_agents_for_team(db, positions, self.id).await;
        free_agents.sort_by(|a, b| b.ability.display().cmp(&a.ability.display()));

        return free_agents;
    }

    // Save the player needs to the database.
    async fn save_player_needs(&self, db: &Db) {
        let json = json!(self.player_needs);

        sqlx::query(
            "UPDATE Team SET player_needs = $1
            WHERE id = $2"
        ).bind(&json)
        .bind(self.id)
        .execute(db).await.unwrap();

        let _player_needs: Vec<PlayerNeed> = serde_json::from_value(json.clone()).expect(format!(
            "null value in player_needs_json\n
            json: {:#?}\n
            vec: {:#?}", json.to_string(), self.player_needs
        ).as_str());
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
fn evaluate_position_needs(position: PositionId, players: Vec<Player>) -> PlayerNeed {
    let mut need = PlayerNeed::build(position);
    let players_in_lineup = players.len().clamp(0, need.lineup_places() as usize);
    need.abilities = players[0..players_in_lineup].into_iter().map(|a| a.ability.display() as f64).collect();

    return need;
}