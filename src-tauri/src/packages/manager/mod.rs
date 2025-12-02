use std::collections::HashMap;

use crate::logic::{person::player::position::PositionId, team::ai::PlayerNeed, time::AnnualWindow};

// This struct is used when handling daily simulation.
struct Manager {
    pub person: Person,
    pub is_human: bool,
}

struct Person {
    team: Option<Team>,
}

struct Team {
    players_and_approached: HashMap<PositionId, Vec<Player>>,
    player_needs: Vec<PlayerNeed>,
    actions_remaining: u8,
    season_window: AnnualWindow,
}

struct Player {
    ability: u8,
}