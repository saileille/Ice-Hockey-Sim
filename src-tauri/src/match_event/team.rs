use weighted_rand::{builder::{NewBuilder, WalkerTableBuilder}, table::WalkerTable};

use super::event::Shot;

use crate::{person::player::Player, team::{Team, lineup::{LineUp, DefencePair, ForwardLine}}};

#[derive(Default, Clone)]
pub struct TeamData {
    pub team_id: usize,
    pub shots: Vec<Shot>,
    pub lineup: LineUp,
    pub players_on_ice: PlayersOnIce,
    penalties: Vec<String>, // Placeholder.
}

impl TeamData { // Basics.
    pub fn new(team_id: usize) -> Self {
        let mut team_data: TeamData = TeamData::default();
        team_data.team_id = team_id;
        return team_data;
    }

    // Make sure the TeamData does not contain illegal values.
    pub fn is_valid(&self) -> bool {
        self.team_id != 0
    }
    
    // Get a clone of the team.
    pub fn get_team_clone(&self) -> Team {
        Team::fetch_from_db(&self.team_id)
    }
}

impl TeamData {
    fn get_shot_amount(&self) -> u16 {
        self.shots.len() as u16
    }

    pub fn get_goal_amount(&self) -> u16 {
        let mut goal_counter: u16 = 0;
        for shot in self.shots.iter() {
            if shot.is_goal {
                goal_counter += 1;
            }
        }

        return goal_counter;
    }

    // Reset the TeamData.
    pub fn reset(&mut self) {
        self.shots = Vec::new();
        self.players_on_ice = PlayersOnIce::new();
        self.penalties = Vec::new();
    }

    // Determine who should go on ice next.
    pub fn change_players_on_ice(&mut self) {
        // The better goalkeeper is always on ice (for now).
        let mut goalkeepers: [Player; 2] = self.lineup.get_goalkeeper_clones();
        goalkeepers.sort_by(|a, b| b.ability.cmp(&a.ability));

        self.players_on_ice.gk_id = goalkeepers[0].id;

        // Simple randomness to determine which line is playing.
        // This should be player-editable in the future.
        // 1st line: 40%, 2nd line: 30%, 3rd line: 20%, 4th line: 40%
        let weights: [u32; 4] = [4, 3, 2, 1];
        let builder: WalkerTableBuilder = WalkerTableBuilder::new(&weights);
        let wa_table: WalkerTable = builder.build();

        let index: usize = wa_table.next();
        
        let d_pair: DefencePair = self.lineup.defence_pairs[index].clone();
        self.players_on_ice.ld_id = d_pair.ld_id;
        self.players_on_ice.rd_id = d_pair.rd_id;
        
        let f_line: ForwardLine = self.lineup.forward_lines[index].clone();
        self.players_on_ice.lw_id = f_line.lw_id;
        self.players_on_ice.c_id = f_line.c_id;
        self.players_on_ice.rw_id = f_line.rw_id;
    }
}

#[derive(Default, Clone)]
pub struct PlayersOnIce {
    pub gk_id: usize,
    pub ld_id: usize,
    pub rd_id: usize,
    pub lw_id: usize,
    pub c_id: usize,
    pub rw_id: usize,
    extra_attacker_id: usize,
}

impl PlayersOnIce {
    pub fn new() -> Self {
        PlayersOnIce::default()
    }

    // Get PlayersOnIceClones struct.
    pub fn get_player_clones(&self) -> PlayersOnIceClones {
        PlayersOnIceClones::new(self)
    }

    pub fn get_goalkeeper_clone(&self) -> Player {
        Player::fetch_from_db(&self.gk_id)
    }

    fn get_left_defender_clone(&self) -> Player {
        Player::fetch_from_db(&self.ld_id)
    }

    fn get_right_defender_clone(&self) -> Player {
        Player::fetch_from_db(&self.rd_id)
    }

    fn get_left_winger_clone(&self) -> Player {
        Player::fetch_from_db(&self.lw_id)
    }

    fn get_centre_clone(&self) -> Player {
        Player::fetch_from_db(&self.c_id)
    }

    fn get_right_winger_clone(&self) -> Player {
        Player::fetch_from_db(&self.rw_id)
    }
}

impl PlayersOnIce {
    fn count(&self) -> u8 {
        // Count how many players are on ice.
        let mut counter: u8 = 0;
        if self.gk_id != 0 {
            counter += 1;
        } if self.ld_id != 0 {
            counter += 1;
        } if self.rd_id != 0 {
            counter += 1;
        } if self.lw_id != 0 {
            counter += 1;
        } if self.c_id != 0 {
            counter += 1;
        } if self.rw_id != 0 {
            counter += 1;
        } if self.extra_attacker_id != 0 {
            counter += 1;
        }
        return counter;
    }
}

#[derive(Default)]
pub struct PlayersOnIceClones {
    gk: Player,
    ld: Player,
    rd: Player,
    lw: Player,
    c: Player,
    rw: Player,
    extra_attacker: Player,
}

impl PlayersOnIceClones {   // Basics.
    fn new(players_on_ice: &PlayersOnIce) -> Self {
        PlayersOnIceClones {
            gk: Player::fetch_from_db(&players_on_ice.gk_id),
            ld: Player::fetch_from_db(&players_on_ice.ld_id),
            rd: Player::fetch_from_db(&players_on_ice.rd_id),
            lw: Player::fetch_from_db(&players_on_ice.lw_id),
            c: Player::fetch_from_db(&players_on_ice.c_id),
            rw: Player::fetch_from_db(&players_on_ice.rw_id),
            extra_attacker: Player::fetch_from_db(&players_on_ice.extra_attacker_id),
        }
    }
}

impl PlayersOnIceClones {
    // Get the total ability of skaters (not goalkeeper).
    fn get_skaters_ability(&self) -> u16 {
        let mut total_ability: u16 = 0;

        if self.ld.is_valid() {
            total_ability += self.ld.ability as u16;
        } if self.rd.is_valid() {
            total_ability += self.rd.ability as u16;
        } if self.lw.is_valid() {
            total_ability += self.rd.ability as u16;
        } if self.c.is_valid() {
            total_ability += self.c.ability as u16;
        } if self.rw.is_valid() {
            total_ability += self.rw.ability as u16;
        } if self.extra_attacker.is_valid() {
            total_ability += self.extra_attacker.ability as u16;
        }

        return total_ability;
    }

    // Compare the ability of skaters on ice to the opponent.
    pub fn get_skaters_ability_ratio(&self, opponent: PlayersOnIceClones) -> f64 {
        let ability: f64 = self.get_skaters_ability() as f64;
        let both_sides_ability: f64 = ability + (opponent.get_skaters_ability() as f64);

        // To avoid dividing by zero.
        match both_sides_ability {
            0.0 => return 0.5,
            _ => return ability / both_sides_ability
        }
    }

    // Get the valid skaters in a vector.
    pub fn get_skaters_in_vector(&self) -> Vec<Player> {
        let mut players: Vec<Player> = Vec::new();

        if self.ld.is_valid() {players.push(self.ld.clone())}
        if self.rd.is_valid() {players.push(self.rd.clone())}
        if self.lw.is_valid() {players.push(self.lw.clone())}
        if self.c.is_valid() {players.push(self.c.clone())}
        if self.rw.is_valid() {players.push(self.rw.clone())}

        return players;
    }
}