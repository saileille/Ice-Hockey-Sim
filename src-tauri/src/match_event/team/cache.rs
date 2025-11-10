// Team data cache.

use crate::{match_event::team::{TeamGameData}, team::{lineup::{cache::LineUpCache, LineUp}, Team}};

#[derive(Debug)]
#[derive(Default, Clone)]
pub struct TeamGameDataCache {
    pub team: Team,
    pub lineup: LineUpCache,
}

impl TeamGameDataCache {
    pub fn build(team_game_data: &TeamGameData) -> Self {
        Self {
            team: Team::fetch_from_db(&team_game_data.team_id),
            ..Default::default()
        }
    }

    pub fn build_lineup(&mut self, lineup: &LineUp) {
        self.lineup = LineUpCache::build(lineup);
    }
}
