// Knockout season parametres.

use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{Decode, Encode, Sqlite, encode::IsNull, error::BoxDynError, sqlite::{SqliteArgumentValue, SqliteTypeInfo, SqliteValueRef}};
use time::Date;

use crate::{competition::{Competition, season::{schedule_generator::assign_dates, team::TeamSeason}}, types::{Db, SeasonId, TeamId}};

#[derive(Debug, Serialize, Deserialize)]
#[derive(Default, Clone)]
pub struct KnockoutRound {
    pub pairs: Vec<KnockoutPair>,
    pub advanced_teams: Vec<TeamSeason>,
    pub eliminated_teams: Vec<TeamSeason>,
}

impl sqlx::Type<Sqlite> for KnockoutRound {
    fn type_info() -> SqliteTypeInfo {
        <String as sqlx::Type<Sqlite>>::type_info()
    }
}

impl<'q> Encode<'q, Sqlite> for KnockoutRound {
    fn encode(self, buf: &mut Vec<SqliteArgumentValue<'q>>) -> Result<IsNull, BoxDynError> {
        Encode::<Sqlite>::encode(serde_json::to_string(&self).unwrap(), buf)
    }

    fn encode_by_ref(&self, buf: &mut Vec<SqliteArgumentValue<'q>>) -> Result<IsNull, BoxDynError> {
        Encode::<Sqlite>::encode(serde_json::to_string(self).unwrap(), buf)
    }
}

impl<'r> Decode<'r, Sqlite> for KnockoutRound {
    fn decode(value: SqliteValueRef<'r>) -> Result<Self, BoxDynError> {
        let json = <serde_json::Value as Decode<Sqlite>>::decode(value)?;
        Ok(serde_json::from_value(json)?)
    }
}

impl KnockoutRound {
    // Build it.
    pub fn build() -> Self {
        Self::default()
    }

    async fn save(&self, db: &Db, season_id: SeasonId) {
        sqlx::query(
            "UPDATE Season SET knockout_round = $1 WHERE id = $2"
        ).bind(self)
        .bind(season_id)
        .execute(db).await.unwrap();
    }

    // Get relevant information for a competition screen.
    pub async fn get_comp_screen_json(&self, db: &Db) -> serde_json::Value {
        let mut pairs = Vec::new();
        for pair in self.pairs.iter() {
            pairs.push(pair.get_comp_screen_json(db).await);
        }

        json!({
            "pairs": pairs
        })
    }

    // Set up a knockout round.
    pub async fn setup(&mut self, db: &Db, teams: &[TeamSeason], start: Date, end: Date, comp: &Competition, season_id: SeasonId) {
        self.draw_teams(db, teams, season_id).await;
        let matchdays = self.generate_matchdays(comp);
        assign_dates(db, matchdays, start, end, comp, false).await;
    }

    // Draw the pairs for the round.
    async fn draw_teams(&mut self, db: &Db, teams: &[TeamSeason], season_id: SeasonId) {
        let mut pots = self.create_pots_and_pairs(teams);

        for pair in self.pairs.iter_mut() {
            let last_index = pots.len() - 1;

            let mut draw_pots = if pots.len() > 1 {
                vec![pots[0].clone(), pots[last_index].clone()]
            }
            else {
                vec![pots[0].clone()]
            };

            // Draw the teams for the pair.
            let home_id = Self::draw_team(&mut draw_pots.first_mut().unwrap().1);
            let away_id = Self::draw_team(&mut draw_pots.last_mut().unwrap().1);

            pair.home = TeamSeason::build(home_id, draw_pots.first().unwrap().0);
            pair.away = TeamSeason::build(away_id, draw_pots.last().unwrap().0);

            // Remove pots if empty.
            for (i, pot) in draw_pots.into_iter().rev().enumerate() {
                let index = match i {
                    0 => last_index,
                    _ => 0,
                };

                if pot.1.is_empty() {
                    pots.remove(index);
                }
                else {
                    pots[index] = pot
                }
            }
        }

        self.save(db, season_id).await;
    }

    // Create pots from which to draw teams. Top seeds are first, bottom seeds are last.
    fn create_pots_and_pairs(&mut self, teams: &[TeamSeason]) -> Vec<(u8, Vec<TeamId>)> {
        for _ in 0..teams.len() / 2 {
            self.pairs.push(KnockoutPair::default());
        }

        let mut pots: Vec<(u8, Vec<u8>)> = Vec::new();
        for team in teams.iter() {
            match pots.iter().position(|pot| pot.0 == team.seed) {
                // Add team to an existing pot.
                Some(i) => pots[i].1.push(team.team_id),
                // Create a new pot if one does not exist.
                _ => pots.push((team.seed, vec![team.team_id]))
            }
        }

        // Sorting by seeds.
        pots.sort_by(|a, b| a.0.cmp(&b.0));
        return pots;
    }

    // Draw a team from the pot, and remove it from the pot.
    fn draw_team(pot: &mut Vec<TeamId>) -> TeamId {
        pot.swap_remove(rand::random_range(0..pot.len()))
    }

    // Update the teamdata for the knockout pairs.
    pub async fn update_teamdata(&mut self, db: &Db, home: &TeamSeason, away: &TeamSeason, season_id: SeasonId) {
        for pair in self.pairs.iter_mut() {
            if !pair.is_over {
                pair.update_teamdata(home, away);
            }
        }
        self.save(db, season_id).await;
    }

    // Check if the knockout round is over.
    pub async fn check_if_over(&mut self, db: &Db, comp: &Competition) -> bool {
        let mut is_over = true;
        for pair in self.pairs.iter_mut() {
            if pair.is_over { continue; }

            let is_pair_over = pair.get_winner_loser(comp.format.as_ref().unwrap().knockout_round.as_ref().unwrap().wins_required);
            if is_pair_over.is_none() {
                is_over = false;
                continue;
            }

            pair.is_over = true;
            pair.clean_up_games(db).await;
            let teams = is_pair_over.unwrap();
            self.advanced_teams.push(teams[0].clone());
            self.eliminated_teams.push(teams[1].clone());
        }

        return is_over;
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[derive(Default, Clone)]
pub struct KnockoutPair {
    pub home: TeamSeason,
    pub away: TeamSeason,
    is_over: bool,
}

// Basics.
impl KnockoutPair {
    // Get nice JSON for comp screen.
    async fn get_comp_screen_json(&self, db: &Db) -> serde_json::Value {
        json!({
            "home": self.home.comp_screen_package_pair(db).await,
            "away": self.away.comp_screen_package_pair(db).await,
        })
    }

    // Get the victor and the loser of the pair, or None if neither has won.
    fn get_winner_loser(&self, wins_required: u8) -> Option<[TeamSeason; 2]> {
        if self.home.all_wins() >= wins_required {
            return Some([self.home.clone(), self.away.clone()]);
        }
        if self.away.all_wins() >= wins_required {
            return Some([self.away.clone(), self.home.clone()]);
        }

        return None;
    }

    // Remove any upcoming games from these two teams.
    async fn clean_up_games(&self, db: &Db) {
        sqlx::query(
            "DELETE FROM Game
            WHERE (home_id = $1 OR away_id = $1 OR home_id = $2 OR away_id = $2)
            AND unixepoch(date) > (
                SELECT unixepoch(value_data) FROM KeyValue
                WHERE key_name = 'today'
            )"
        ).bind(self.home.team_id)
        .bind(self.away.team_id)
        .execute(db).await.unwrap();
    }

    // Update the teamdata for the pair.
    fn update_teamdata(&mut self, home: &TeamSeason, away: &TeamSeason) {
        if self.home.team_id == home.team_id {
            self.home.update(home);
            self.away.update(away);
        }
        else if self.home.team_id == away.team_id {
            self.home.update(away);
            self.away.update(home);
        }


    }
}