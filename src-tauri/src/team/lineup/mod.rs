pub mod cache;

use crate::{
    types::PlayerId,
    person::player::{
        Player,
        position::PositionId
    }
};

// A line-up of players used in a match.
#[derive(Debug, serde::Serialize)]
#[derive(Default, Clone)]
pub struct LineUp {
    gk_ids: [PlayerId; 2],
    pub defence_pairs: [DefencePair; 4],
    pub forward_lines: [ForwardLine; 4],
}

impl LineUp {
    // Make sure the lineup is filled.
    pub fn is_full(&self) -> bool {
        if self.gk_ids.contains(&0) { return false; }

        for pair in self.defence_pairs.iter() {
            if !pair.is_full() { return false; }
        }

        for line in self.forward_lines.iter() {
            if !line.is_full() { return false; }
        }

        return true;
    }
}

impl LineUp {
    // Clear the lineup.
    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

impl LineUp {
    // Add players from a roster to the lineup.
    pub fn auto_add(&mut self, players: Vec<Player>) {
        for player in players {
            self.auto_add_player(player);
        }
    }

    // Add a player to the lineup.
    fn auto_add_player(&mut self, player: Player) {
        match player.position_id {
            PositionId::Goalkeeper => self.auto_add_gk(player),
            PositionId::LeftDefender => self.auto_add_ld(player),
            PositionId::RightDefender => self.auto_add_rd(player),
            PositionId::LeftWinger => self.auto_add_lw(player),
            PositionId::Centre => self.auto_add_c(player),
            PositionId::RightWinger => self.auto_add_rw(player),
            _ => return
        }
    }

    // Add a goalkeeper to the lineup.
    fn auto_add_gk(&mut self, player: Player) {
        for id in self.gk_ids.iter_mut() {
            if *id == 0 {
                *id = player.id;
                return;
            }
        }
    }

    // Add a left defender to the lineup.
    fn auto_add_ld(&mut self, player: Player) {
        for pair in self.defence_pairs.iter_mut() {
            if pair.ld_id == 0 {
                pair.ld_id = player.id;
                return;
            }
        }
    }

    // Add a left defender to the lineup.
    fn auto_add_rd(&mut self, player: Player) {
        for pair in self.defence_pairs.iter_mut() {
            if pair.rd_id == 0 {
                pair.rd_id = player.id;
                return;
            }
        }
    }

    // Add a left winger to the lineup.
    fn auto_add_lw(&mut self, player: Player) {
        for line in self.forward_lines.iter_mut() {
            if line.lw_id == 0 {
                line.lw_id = player.id;
                return;
            }
        }
    }

    // Add a centre to the lineup.
    fn auto_add_c(&mut self, player: Player) {
        for line in self.forward_lines.iter_mut() {
            if line.c_id == 0 {
                line.c_id = player.id;
                return;
            }
        }
    }

    // Add a right winger to the lineup.
    fn auto_add_rw(&mut self, player: Player) {
        for line in self.forward_lines.iter_mut() {
            if line.rw_id == 0 {
                line.rw_id = player.id;
                return;
            }
        }
    }
}

// A pair of defenders used in a line-up.
#[derive(Debug, serde::Serialize)]
#[derive(Default, Clone)]
pub struct DefencePair {
    pub ld_id: PlayerId,
    pub rd_id: PlayerId,
}

impl DefencePair {  // Basics.
    // Get a clone of the left defender.
    fn get_left_defender(&self) -> Option<Player> {
        Player::fetch_from_db(&self.ld_id)
    }

    // Get a clone of the right defender.
    fn get_right_defender(&self) -> Option<Player> {
        Player::fetch_from_db(&self.rd_id)
    }

    // Make sure the defence pair is full.
    fn is_full(&self) -> bool {
        self.ld_id != 0 &&
        self.rd_id != 0
    }
}

impl DefencePair {
    // Clear the defence pair.
    fn clear(&mut self) {
        self.ld_id = 0;
        self.rd_id = 0;
    }
}

// A line of forwards used in a line-up.
#[derive(Debug, serde::Serialize)]
#[derive(Default, Clone)]
pub struct ForwardLine {
    pub lw_id: PlayerId,
    pub c_id: PlayerId,
    pub rw_id: PlayerId,
}

impl ForwardLine {  // Basics.
    // Get a clone of the left winger.
    fn get_left_winger(&self) -> Option<Player> {
        Player::fetch_from_db(&self.lw_id)
    }

    // Get a clone of the centre forward.
    fn get_centre(&self) -> Option<Player> {
        Player::fetch_from_db(&self.c_id)
    }

    // Get a clone of the right winger.
    fn get_right_winger(&self) -> Option<Player> {
        Player::fetch_from_db(&self.rw_id)
    }

    // Make sure the forward line is full.
    fn is_full(&self) -> bool {
        self.lw_id != 0 &&
        self.c_id != 0 &&
        self.rw_id != 0
    }
}

impl ForwardLine {
    // Clear the forward line.
    fn clear(&mut self) {
        self.lw_id = 0;
        self.c_id = 0;
        self.rw_id = 0;
    }
}