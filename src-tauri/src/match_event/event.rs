// An event is anything worth of writing down that happens during a match.
// Shot, goal, penalty, etc.
use rand::{rngs::ThreadRng, seq::IndexedRandom};
use crate::{event, match_event::Clock, person::player::Player, team::lineup::cache::PlayersOnIceCache, types::PlayerId};

#[derive(Debug)]
#[derive(Default, Clone)]
pub struct PlayersOnIce {
    gk: PlayerId,
    ld: PlayerId,
    rd: PlayerId,
    lw: PlayerId,
    c: PlayerId,
    rw: PlayerId,
    extra_attacker: PlayerId,
}

impl PlayersOnIce {
    pub fn build(gk: PlayerId, ld: PlayerId, rd: PlayerId, lw: PlayerId, c: PlayerId, rw: PlayerId, extra_attacker: PlayerId) -> Self {
        Self {
            gk: gk,
            ld: ld,
            rd: rd,
            lw: lw,
            c: c,
            rw: rw,
            extra_attacker: extra_attacker,
        }
    }
}

#[derive(Debug)]
#[derive(Default, Clone)]
pub struct Event {
    pub time: Clock,
    attacking_players: PlayersOnIce,
    defending_players: PlayersOnIce,
}

impl Event {
    fn build(time: Clock, attacking_players: PlayersOnIce, defending_players: PlayersOnIce) -> Self {
        Self {
            time: time,
            attacking_players: attacking_players,
            defending_players: defending_players,
        }
    }
}

#[derive(Debug)]
#[derive(Default, Clone)]
pub struct Shot {
    pub event: Event,
    pub is_goal: bool,
    shooter_id: PlayerId,
    assister_ids: Vec<PlayerId>,
}

// Basics.
impl Shot {
    fn build(time: Clock, attacking_players: PlayersOnIce, defending_players: PlayersOnIce) -> Self {
        Self {
            event: Event::build(time, attacking_players, defending_players),
            ..Default::default()
        }
    }

    // Do the building, calculating, simulating, everything, here.
    pub fn simulate(time: Clock, attackers: &PlayersOnIceCache, defenders: &PlayersOnIceCache, rng: &mut ThreadRng) -> Self {
        let attacking_ids = attackers.get_ids();
        let defending_ids = defenders.get_ids();
        let mut shot = Self::build(time, attacking_ids, defending_ids);

        let shooter_and_assisters = shot.create_shooter_and_assisters(attackers, rng);
        shot.calculate_goal(&shooter_and_assisters, defenders, rng);

        return shot;
    }

    // Get shooter object.
    fn get_shooter(&self) -> Player {
        Player::fetch_from_db(&self.shooter_id).unwrap()
    }

    // Get assister objects.
    fn get_assisters(&self) -> Vec<Player> {
        let mut clones = Vec::new();

        for id in self.assister_ids.iter() {
            let player = Player::fetch_from_db(&id);
            if !player.is_none() {
                clones.push(player.unwrap())
            }
        }
        return clones;
    }
}

impl Shot {
    // Determine who shoots and who assists.
    // Completely random for now.
    fn create_shooter_and_assisters(&mut self, attackers: &PlayersOnIceCache, rng: &mut ThreadRng) -> Vec<Player> {
        let players = attackers.create_vector_of_skaters();
        let mut shooter_and_assisters_ids = Vec::new();

        let mut shooter_and_assisters = Vec::new();

        for i in 0..3 {
            let chosen = players.choose(rng)
                .expect(&format!("could not choose Player. iteration: {i}, players.len(): {}, players on ice: {attackers:#?}", players.len()));

            let id = chosen.id;


            if shooter_and_assisters_ids.contains(&id) {
                break;
            }
            else {
                shooter_and_assisters_ids.push(id);
                shooter_and_assisters.push(chosen.clone());
            }
        }

        for id in shooter_and_assisters_ids {
            if self.shooter_id == 0 {
                self.shooter_id = id;
            }
            else {
                self.assister_ids.push(id);
            }
        }

        return shooter_and_assisters;
    }

    // Check if the shot ends up in goal.
    // Only taking shooter into account for now.
    fn calculate_goal(&mut self, shooter_and_assisters: &[Player], defenders: &PlayersOnIceCache, rng: &mut ThreadRng) {
        let gk_ability = defenders.gk.as_ref().unwrap().ability.get() as f64;
        let shooter_ability = shooter_and_assisters[0].ability.get() as f64;
        let total_ability = gk_ability + shooter_ability;
        let modifier;

        if total_ability == 0.0 { modifier = 0.5 }
        else { modifier = shooter_ability / total_ability }

        if event::Type::fetch_from_db(&event::Id::Goal).get_outcome(modifier, rng) {
            self.is_goal = true;
        }
    }
}

impl Shot { // Testing stuff.
    pub fn scorer_and_assists_to_string(&self) -> String {
        let string = self.get_shooter().person.get_full_name();

        let mut assisters_string = String::new();
        let assisters = self.get_assisters();

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