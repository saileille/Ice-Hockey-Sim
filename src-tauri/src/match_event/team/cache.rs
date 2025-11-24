// Team data cache.

use crate::{match_event::team::TeamGame, team::{Team, lineup::{cache::LineUpCache}}, types::{Db, GameId, TeamId}};

#[derive(Debug)]
#[derive(Default, Clone)]
pub struct TeamGameCache {
    pub team: Team,
    pub game_data: TeamGame,
    pub lineup: LineUpCache,
}

impl TeamGameCache {
    pub async fn build(db: &Db, game_id: GameId, id: TeamId) -> Self {
        Self {
            team: Team::fetch_from_db(db, id).await,
            game_data: TeamGame::fetch_from_db(db, game_id, id).await,

            ..Default::default()
        }
    }

    pub async fn build_lineup(&mut self, db: &Db) {
        self.lineup = LineUpCache::build(db, &self.team.lineup).await;
    }
}
