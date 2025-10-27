// Lineup cache stuff.

use crate::{match_event::event::PlayersOnIce, misc::random_with_weights, person::player::Player, team::lineup::{DefencePair, ForwardLine, LineUp}, types::PlayerId};

#[derive(Debug)]
#[derive(Default, Clone)]
pub struct LineUpCache {
    goalkeepers: [Option<Player>; 2],
    defence_pairs: [DefencePairCache; 4],
    forward_lines: [ForwardLineCache; 4],
    pub players_on_ice: PlayersOnIceCache,
}

impl LineUpCache {
    pub fn build(lineup: &LineUp) -> Self {
        let mut cache = Self::default();

        for (i, gk) in lineup.gk_ids.iter().enumerate() {
            cache.goalkeepers[i] = Player::fetch_from_db(gk);
        }

        for (i, pair) in lineup.defence_pairs.iter().enumerate() {
            cache.defence_pairs[i] = DefencePairCache::build(pair);
        }

        for (i, line) in lineup.forward_lines.iter().enumerate() {
            cache.forward_lines[i] = ForwardLineCache::build(line);
        }

        return cache;
    }

    // Determine who should go on ice next.
    pub fn change_players_on_ice(&mut self) {
        self.players_on_ice = PlayersOnIceCache::default();

        // The better goalkeeper is always on ice (for now).
        self.players_on_ice.gk = self.goalkeepers[0].clone();

        // Simple randomness to determine which line is playing.
        // This should be player-editable in the future.
        // 1st line: 40%, 2nd line: 30%, 3rd line: 20%, 4th line: 10%
        let index = random_with_weights(&[4, 3, 2, 1], None, None);

        self.players_on_ice.ld = self.defence_pairs[index].ld.clone();
        self.players_on_ice.rd = self.defence_pairs[index].rd.clone();
        self.players_on_ice.lw = self.forward_lines[index].lw.clone();
        self.players_on_ice.c = self.forward_lines[index].c.clone();
        self.players_on_ice.rw = self.forward_lines[index].rw.clone();
    }

    // Get the average ability of the lineup.
    // This is for player contract AI.
    pub fn get_average_ability(&self) -> f64 {
        let mut total_ability = 0;
        let mut counter: u8 = 0;

        for gk in self.goalkeepers.iter() {
            if gk.is_some() {
                total_ability += gk.as_ref().unwrap().ability as u16;
                counter += 1;
            }
        }

        for pair in self.defence_pairs.iter() {
            if pair.ld.is_some() {
                total_ability += pair.ld.as_ref().unwrap().ability as u16;
                counter += 1;
            }
            if pair.rd.is_some() {
                total_ability += pair.rd.as_ref().unwrap().ability as u16;
                counter += 1;
            }
        }

        for line in self.forward_lines.iter() {
            if line.lw.is_some() {
                total_ability += line.lw.as_ref().unwrap().ability as u16;
                counter += 1;
            }
            if line.c.is_some() {
                total_ability += line.c.as_ref().unwrap().ability as u16;
                counter += 1;
            }
            if line.rw.is_some() {
                total_ability += line.rw.as_ref().unwrap().ability as u16;
                counter += 1;
            }
        }

        match counter {
            0 => 0.0,
            n => (total_ability as f64) / (n as f64)
        }

    }
}

#[derive(Debug)]
#[derive(Default, Clone)]
struct DefencePairCache {
    ld: Option<Player>,
    rd: Option<Player>,
}

impl DefencePairCache {
    fn build(defence_pair: &DefencePair) -> Self {
        DefencePairCache {
            ld: defence_pair.get_left_defender(),
            rd: defence_pair.get_right_defender(),
        }
    }
}

#[derive(Debug)]
#[derive(Default, Clone)]
struct ForwardLineCache {
    lw: Option<Player>,
    c: Option<Player>,
    rw: Option<Player>,
}

impl ForwardLineCache {
    fn build(forward_line: &ForwardLine) -> Self {
        ForwardLineCache {
            lw: forward_line.get_left_winger(),
            c: forward_line.get_centre(),
            rw: forward_line.get_right_winger(),
        }
    }
}

#[derive(Debug)]
#[derive(Default, Clone)]
pub struct PlayersOnIceCache {
    pub gk: Option<Player>,
    ld: Option<Player>,
    rd: Option<Player>,
    lw: Option<Player>,
    c: Option<Player>,
    rw: Option<Player>,
    extra_attacker: Option<Player>,
}

impl PlayersOnIceCache {
    // Get the total ability of skaters (not goalkeeper).
    fn get_skaters_ability(&self) -> u16 {
        let mut total_ability = 0;

        if self.ld.is_some() { total_ability += self.ld.as_ref().unwrap().ability as u16 }
        if self.rd.is_some() { total_ability += self.rd.as_ref().unwrap().ability as u16 }
        if self.lw.is_some() { total_ability += self.lw.as_ref().unwrap().ability as u16 }
        if self.c.is_some() { total_ability += self.c.as_ref().unwrap().ability as u16 }
        if self.rw.is_some() { total_ability += self.rw.as_ref().unwrap().ability as u16 }
        if self.extra_attacker.is_some() { total_ability += self.extra_attacker.as_ref().unwrap().ability as u16 }

        return total_ability;
    }

    // Compare the ability of skaters on ice to the opponent.
    pub fn get_skaters_ability_ratio(&self, opponent: &Self) -> f64 {
        let ability = self.get_skaters_ability() as f64;
        let both_sides_ability = ability + (opponent.get_skaters_ability() as f64);

        // To avoid dividing by zero.
        match both_sides_ability {
            0.0 => return 0.5,
            _ => return ability / both_sides_ability
        }
    }

    // Get the IDs of the players.
    pub fn get_ids(&self) -> PlayersOnIce {
        PlayersOnIce::build(
            Self::get_player_id(&self.gk.as_ref()),
            Self::get_player_id(&self.ld.as_ref()),
            Self::get_player_id(&self.rd.as_ref()),
            Self::get_player_id(&self.lw.as_ref()),
            Self::get_player_id(&self.c.as_ref()),
            Self::get_player_id(&self.rw.as_ref()),
            Self::get_player_id(&self.extra_attacker.as_ref())
        )
    }

    // Get a player's ID.
    fn get_player_id(player: &Option<&Player>) -> PlayerId {
        if player.is_none() { return 0; }
        return player.unwrap().id;
    }

    // Create a vector of the skaters.
    pub fn create_vector_of_skaters(&self) -> Vec<Player> {
        let mut vector = Vec::new();
        if self.ld.is_some() { vector.push(self.ld.as_ref().unwrap().clone()); }
        if self.rd.is_some() { vector.push(self.rd.as_ref().unwrap().clone()); }
        if self.lw.is_some() { vector.push(self.lw.as_ref().unwrap().clone()); }
        if self.c.is_some() { vector.push(self.c.as_ref().unwrap().clone()); }
        if self.rw.is_some() { vector.push(self.rw.as_ref().unwrap().clone()); }
        if self.extra_attacker.is_some() { vector.push(self.extra_attacker.as_ref().unwrap().clone()); }

        return vector;
    }
}
