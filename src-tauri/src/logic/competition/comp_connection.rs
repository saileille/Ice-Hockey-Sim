use sqlx::FromRow;

use crate::logic::{competition::{Seed, season::team::TeamSeason}, types::{CompetitionId, Db}};


// Stores data for which teams to go to which competition.
#[derive(Debug, Default, Clone)]
#[derive(FromRow)]
pub struct CompConnection {
    pub origin_id: CompetitionId,
    pub destination_id: CompetitionId,
    pub highest_position: u8,
    pub lowest_position: u8,
    pub team_seeds: Seed,
    pub stats_carry_over: bool,
}

impl CompConnection {
    // Build the element.
    pub fn build(origin_id: CompetitionId, highest_position: u8, lowest_position: u8, team_seeds: Seed, stats_carry_over: bool) -> Self {
        Self {
            origin_id,
            highest_position,
            lowest_position,
            team_seeds,
            stats_carry_over,

            ..Default::default()
        }
    }

    // Send teams onwards to the next stage.
    pub async fn send_teams(&self, db: &Db, teams: &[TeamSeason]) {
        let mut teamdata = (self.highest_position - 1..self.lowest_position).map(|i| {
            let seed = match self.team_seeds {
                Seed::GetFromPosition => i + 1,
                Seed::Preserve => teams[i as usize].seed,
                _ => panic!(),
            };

            let team = if self.stats_carry_over {
                let mut t = teams[i as usize].clone();
                t.seed = seed;
                t
            } else {
                TeamSeason::build(teams[i as usize].team_id, seed)
            };
            team
        }).collect();

        self.destination(db).await.setup_season(db, &mut teamdata).await;
    }
}