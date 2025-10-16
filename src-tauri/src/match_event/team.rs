use weighted_rand::{
    builder::{NewBuilder, WalkerTableBuilder}
};

use crate::{
    competition::season::team::TeamCompData, person::player::Player, team::{
        lineup::LineUp,
        Team
    }, types::{convert, PlayerId, TeamId}
};
use super::event::Shot;

#[derive(Debug, serde::Serialize)]
#[derive(Default, Clone)]
pub struct TeamGameData {
    pub team_id: TeamId,
    pub team_seed: u8,
    pub shots: Vec<Shot>,
    pub lineup: LineUp,
    pub players_on_ice: Option<PlayersOnIce>,
    penalties: Vec<String>, // Placeholder.
}

impl TeamGameData { // Basics.
    pub fn build(team: &TeamCompData) -> Self {
        let mut team_data = TeamGameData::default();
        team_data.team_id = team.team_id;
        team_data.team_seed = team.seed;
        return team_data;
    }

    // Make sure the TeamData does not contain illegal values.
    pub fn is_valid(&self) -> bool {
        self.team_id != 0
    }

    // Get a clone of the team.
    pub fn get_team(&self) -> Team {
        Team::fetch_from_db(&self.team_id)
    }
}

// Functional.
impl TeamGameData {
    fn get_shot_amount(&self) -> u16 {
        convert::usize_to_u16(self.shots.len())
    }

    pub fn get_goal_amount(&self) -> u16 {
        let mut goal_counter = 0;
        for shot in self.shots.iter() {
            if shot.is_goal { goal_counter += 1; }
        }
        return goal_counter;
    }

    // Determine who should go on ice next.
    pub fn change_players_on_ice(&mut self) {
        let mut players_on_ice = PlayersOnIce::default();

        // The better goalkeeper is always on ice (for now).
        let mut goalkeepers = self.lineup.get_goalkeepers();
        goalkeepers.sort_by(|a, b| b.ability.cmp(&a.ability));

        players_on_ice.gk_id = goalkeepers[0].id;

        // Simple randomness to determine which line is playing.
        // This should be player-editable in the future.
        // 1st line: 40%, 2nd line: 30%, 3rd line: 20%, 4th line: 40%
        let weights = [4, 3, 2, 1];
        let builder = WalkerTableBuilder::new(&weights);
        let wa_table = builder.build();

        let index = wa_table.next();

        let d_pair = self.lineup.defence_pairs[index].clone();
        players_on_ice.ld_id = d_pair.ld_id;
        players_on_ice.rd_id = d_pair.rd_id;

        let f_line = self.lineup.forward_lines[index].clone();
        players_on_ice.lw_id = f_line.lw_id;
        players_on_ice.c_id = f_line.c_id;
        players_on_ice.rw_id = f_line.rw_id;

        self.players_on_ice = Some(players_on_ice);
    }
}

#[derive(Debug, serde::Serialize)]
#[derive(Default, Clone)]
pub struct PlayersOnIce {
    pub gk_id: PlayerId,
    pub ld_id: PlayerId,
    pub rd_id: PlayerId,
    pub lw_id: PlayerId,
    pub c_id: PlayerId,
    pub rw_id: PlayerId,
    extra_attacker_id: PlayerId,
}

impl PlayersOnIce {
    // Get PlayersOnIceClones struct.
    pub fn get(&self) -> PlayersOnIceClones {
        PlayersOnIceClones::build(self)
    }

    pub fn get_goalkeeper(&self) -> Option<Player> {
        Player::fetch_from_db(&self.gk_id)
    }

    fn get_left_defender(&self) -> Option<Player> {
        Player::fetch_from_db(&self.ld_id)
    }

    fn get_right_defender(&self) -> Option<Player> {
        Player::fetch_from_db(&self.rd_id)
    }

    fn get_left_winger(&self) -> Option<Player> {
        Player::fetch_from_db(&self.lw_id)
    }

    fn get_centre(&self) -> Option<Player> {
        Player::fetch_from_db(&self.c_id)
    }

    fn get_right_winger(&self) -> Option<Player> {
        Player::fetch_from_db(&self.rw_id)
    }

    fn get_extra_attacker(&self) -> Option<Player> {
        Player::fetch_from_db(&self.extra_attacker_id)
    }
}

impl PlayersOnIce {
    fn count(&self) -> u8 {
        // Count how many players are on ice.
        let mut counter = 0;
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

#[derive()]
pub struct PlayersOnIceClones {
    gk: Option<Player>,
    ld: Option<Player>,
    rd: Option<Player>,
    lw: Option<Player>,
    c: Option<Player>,
    rw: Option<Player>,
    extra_attacker: Option<Player>,
}

impl PlayersOnIceClones {   // Basics.
    fn build(players_on_ice: &PlayersOnIce) -> Self {
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
        let mut total_ability = 0;
        let skaters = self.get_skaters_in_vector();

        for skater in skaters.iter() {
            total_ability += skater.ability as u16;
        }

        return total_ability;
    }

    // Compare the ability of skaters on ice to the opponent.
    pub fn get_skaters_ability_ratio(&self, opponent_ids: &PlayersOnIce) -> f64 {
        let opponent = opponent_ids.get();
        let ability = self.get_skaters_ability() as f64;
        let both_sides_ability = ability + (opponent.get_skaters_ability() as f64);

        // To avoid dividing by zero.
        match both_sides_ability {
            0.0 => return 0.5,
            _ => return ability / both_sides_ability
        }
    }

    // Get the valid skaters in a vector.
    pub fn get_skaters_in_vector(&self) -> Vec<Player> {
        let mut players = Vec::new();

        if self.ld.is_some() {players.push(self.ld.as_ref().unwrap().clone())}
        if self.rd.is_some() {players.push(self.rd.as_ref().unwrap().clone())}
        if self.lw.is_some() {players.push(self.lw.as_ref().unwrap().clone())}
        if self.c.is_some() {players.push(self.c.as_ref().unwrap().clone())}
        if self.rw.is_some() {players.push(self.rw.as_ref().unwrap().clone())}
        if self.extra_attacker.is_some() {players.push(self.extra_attacker.as_ref().unwrap().clone())}

        return players;
    }
}