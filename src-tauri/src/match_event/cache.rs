// Game cache.


use crate::{match_event::{Rules, team::cache::TeamGameCache}, types::{Db, GameId, TeamId}};

#[derive(Debug)]
#[derive(Default, Clone)]
pub enum Attacker {
    #[default]
    Null,
    Home,
    Away,
}

#[derive(Debug)]
#[derive(Default, Clone)]
pub struct GameCache {
    pub home: TeamGameCache,
    pub away: TeamGameCache,
    pub rules: Rules,
    pub attacker: Attacker,
    // pub initialised: bool,
}

impl GameCache {
    pub async fn build(db: &Db, game_id: GameId, home_id: TeamId, away_id: TeamId, rules: Rules) -> Self {
        Self {
            home: TeamGameCache::build(db, game_id, home_id).await,
            away: TeamGameCache::build(db, game_id, away_id).await,
            rules,
            // initialised: true,

            ..Default::default()
        }
    }

    pub async fn build_lineups(&mut self, db: &Db) {
        self.home.team.auto_build_lineup(db).await;
        self.away.team.auto_build_lineup(db).await;

        self.home.build_lineup(db).await;
        self.away.build_lineup(db).await;
    }
}