// Game cache.

use crate::{match_event::{self, team::{cache::TeamGameDataCache, TeamGameData}}, team::lineup::LineUp};

#[derive(Debug)]
#[derive(Default, Clone)]
pub struct GameCache {
    pub home: TeamGameDataCache,
    pub away: TeamGameDataCache,
    pub rules: match_event::Rules,
}

impl GameCache {
    pub fn build(home: &TeamGameData, away: &TeamGameData, rules: &match_event::Rules) -> Self {
        let mut cache = Self::default();
        cache.home = TeamGameDataCache::build(home);
        cache.away = TeamGameDataCache::build(away);
        cache.rules = rules.clone();

        return cache;
    }

    pub fn build_lineups(&mut self, home: &LineUp, away: &LineUp) {
        self.home.build_lineup(home);
        self.away.build_lineup(away);
    }
}