// An event is anything worth of writing down that happens during a match.
// Shot, goal, penalty, etc.
use rand::{
    rngs::ThreadRng,
    seq::IndexedRandom
};
use crate::{
    person::player::Player,
    event,
    types::PlayerId
};
use super::{
    Clock,
    team::PlayersOnIce
};

#[derive(Default, Clone)]
pub struct Event {
    pub time: Clock,
    attacking_players: PlayersOnIce,
    defending_players: PlayersOnIce,
}

impl Event {
    fn build(time: Clock, attacking_players: &PlayersOnIce, defending_players: &PlayersOnIce) -> Self {
        let mut event: Event = Event::default();
        event.time = time;
        event.attacking_players = attacking_players.clone();
        event.defending_players = defending_players.clone();
        return event;
    }
}

#[derive(Default, Clone)]
pub struct Shot {
    pub event: Event,
    pub is_goal: bool,
    shooter_id: PlayerId,
    assister_ids: Vec<PlayerId>,
}

impl Shot { // Basics.
    pub fn build(time: Clock, attacking_players: &PlayersOnIce, defending_players: &PlayersOnIce) -> Self {
        let mut shot: Shot = Shot::default();
        shot.event = Event::build(time, attacking_players, defending_players);
        return shot;
    }

    // Get shooter object.
    fn get_shooter_clone(&self) -> Player {
        Player::fetch_from_db(&self.shooter_id).unwrap()
    }

    // Get assister objects.
    fn get_assister_clones(&self) -> Vec<Player> {
        let mut clones: Vec<Player> = Vec::new();

        for id in self.assister_ids.iter() {
            let player: Option<Player> = Player::fetch_from_db(&id);
            if !player.is_none() {
                clones.push(player.unwrap())
            }
        }
        return clones;
    }
}

impl Shot {
    // Completely random way to determine who shoots and who assists.
    pub fn create_shooter_and_assisters(&mut self) {
        let players: Vec<Player> = self.event.attacking_players.get_clones().get_skaters_in_vector();
        let mut shooter_and_assisters: Vec<PlayerId> = Vec::new();
        let mut rng: ThreadRng = rand::rng();
        for i in 0..3 {
            let chosen: &Player = players.choose(&mut rng)
                .expect(&format!("could not choose Player. iteration: {i}, players.len(): {}", players.len()));

            let id: PlayerId = chosen.id;

            if shooter_and_assisters.contains(&id) {
                break;
            }
            else {
                shooter_and_assisters.push(id)
            }
        }

        for id in shooter_and_assisters {
            if self.shooter_id == 0 {
                self.shooter_id = id;
            }
            else {
                self.assister_ids.push(id);
            }
        }
    }

    // Check if the shot ends up in goal.
    // Only taking shooter into account for now.
    pub fn calculate_goal(&mut self) {
        let gk_ability: f64 = self.event.defending_players.get_goalkeeper_clone().unwrap().ability as f64;
        let shooter_ability: f64 = self.get_shooter_clone().ability as f64;
        let total_ability: f64 = gk_ability + shooter_ability;
        let modifier: f64;

        if total_ability == 0.0 { modifier = 0.5 }
        else { modifier = shooter_ability / total_ability }

        if event::Type::fetch_from_db(&event::Id::Goal).get_outcome(modifier) {
            self.is_goal = true;
        }
    }
}

impl Shot { // Testing stuff.
    pub fn scorer_and_assists_to_string(&self) -> String {
        let string: String = self.get_shooter_clone().person.get_full_name();

        let mut assisters_string: String = String::new();
        let assisters: Vec<Player> = self.get_assister_clones();

        for (i, assister) in assisters.iter().enumerate() {
            if i > 0 { assisters_string += ", "; }
            assisters_string += &assister.person.get_full_name();
        }

        if assisters_string.len() > 0 {
            assisters_string = format!(" ({assisters_string})");
        }

        return string + &assisters_string;
    }
}