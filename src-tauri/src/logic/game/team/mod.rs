use serde_json::json;
use sqlx::FromRow;

use crate::logic::{game::event::Shot, person::attribute::PersonAttribute, team::lineup::{LineUp, cache::LineUpCache}, types::{Db, GameId, TeamId}};

#[derive(Debug)]
#[derive(Default, Clone)]
#[derive(FromRow)]
pub struct TeamGame {
    pub game_id: GameId,
    pub team_id: TeamId,
    pub lineup: LineUp,

    #[sqlx(skip)]
    pub shots: Vec<Shot>,
    #[sqlx(skip)]
    pub lineup_cache: LineUpCache,
}

// Basics.
impl TeamGame {
    pub async fn comp_screen_package(&self, db: &Db) -> serde_json::Value {
        json!({
            "id": self.team_id,
            "name": self.team_name(db).await,
            "seed": self.team_seed(db).await,
            //"goals": self.goals()
        })
    }

    pub fn goals(&self) -> u16 {
        let mut goal_counter = 0;
        for shot in self.shots.iter() {
            if shot.is_goal { goal_counter += 1; }
        }
        return goal_counter;
    }

    // Build a lineup for the team from its roster and save to database.
    pub async fn auto_build_lineup(&mut self, db: &Db) {
        self.lineup.clear();

        let mut players = self.players(db).await;
        players.sort_by(|a, b| {
            PersonAttribute::display(b.ability.value)
            .cmp(&PersonAttribute::display(a.ability.value))
        });

        self.lineup.auto_add(players);
        self.lineup.save(self.team_id, db).await;
    }

    pub async fn build_lineup_cache(&mut self, db: &Db) {
        self.lineup_cache = LineUpCache::build(db, &self.lineup).await;
    }
}