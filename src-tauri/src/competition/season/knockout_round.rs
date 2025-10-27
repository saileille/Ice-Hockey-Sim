// Knockout season parametres.

use rand::{rng, rngs::ThreadRng, Rng};
use serde_json::json;

use crate::{competition::{season::{schedule_generator::assign_dates, team::TeamCompData}, Competition}, match_event::Game, time::db_string_to_date, types::TeamId};

#[derive(Debug, serde::Serialize)]
#[derive(Default, Clone)]
pub struct KnockoutRound {
    pub pairs: Vec<KnockoutPair>,
    pub advanced_teams: Vec<TeamCompData>,
    pub eliminated_teams: Vec<TeamCompData>,
}

impl KnockoutRound {
    // Build it.
    pub fn build() -> Self {
        Self::default()
    }

    // Get relevant information for a competition screen.
    pub fn get_comp_screen_json(&self) -> serde_json::Value {
        let pairs: Vec<serde_json::Value> = self.pairs.iter().map(|a | a.get_comp_screen_json()).collect();
        json!({
            "pairs": pairs
        })
    }

    // Set up a knockout round.
    // Return the games.
    pub fn setup(&mut self, teams: &[TeamCompData], start: &str, end: &str, comp: &Competition) -> Vec<Game> {
        self.draw_teams(teams);
        let matchdays = self.generate_matchdays(comp);
        return assign_dates(matchdays, &db_string_to_date(start), &db_string_to_date(end), comp, false);
    }

    // Draw the pairs for the round.
    fn draw_teams(&mut self, teams: &[TeamCompData]) {
        let mut pots = self.create_pots_and_pairs(teams);

        let mut rng = rng();
        for pair in self.pairs.iter_mut() {
            let last_index = pots.len() - 1;

            let mut draw_pots = if pots.len() > 1 {
                vec![pots[0].clone(), pots[last_index].clone()]
            }
            else {
                vec![pots[0].clone()]
            };

            // Draw the teams for the pair.
            let home_id = Self::draw_team(&mut draw_pots.first_mut().unwrap().1, &mut rng);
            let away_id = Self::draw_team(&mut draw_pots.last_mut().unwrap().1, &mut rng);

            pair.home = TeamCompData::build(home_id, draw_pots.first().unwrap().0);
            pair.away = TeamCompData::build(away_id, draw_pots.last().unwrap().0);

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
    }

    // Create pots from which to draw teams. Top seeds are first, bottom seeds are last.
    fn create_pots_and_pairs(&mut self, teams: &[TeamCompData]) -> Vec<(u8, Vec<TeamId>)> {
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
    fn draw_team(pot: &mut Vec<TeamId>, rng: &mut ThreadRng) -> TeamId {
        pot.swap_remove(rng.random_range(0..pot.len()))
    }

    // Update the teamdata for the knockout pairs.
    pub fn update_teamdata(&mut self, games: &[Game]) {
        for pair in self.pairs.iter_mut() {
            if !pair.is_over {
                pair.update_teamdata(games);
            }
        }
    }

    // Check if the knockout round is over.
    pub fn check_if_over(&mut self, comp: &Competition, upcoming_games: &mut Vec<Game>) -> bool {
        let mut is_over = true;
        for pair in self.pairs.iter_mut() {
            if pair.is_over { continue; }

            let is_pair_over = pair.get_winner_loser(comp.format.as_ref().unwrap().knockout_round.as_ref().unwrap().wins_required);
            if is_pair_over.is_none() {
                is_over = false;
                continue;
            }

            pair.is_over = true;
            pair.clean_up_games(upcoming_games);
            let teams = is_pair_over.unwrap();
            self.advanced_teams.push(teams[0].clone());
            self.eliminated_teams.push(teams[1].clone());
        }

        return is_over;
    }
}

#[derive(Debug, serde::Serialize)]
#[derive(Default, Clone)]
pub struct KnockoutPair {
    pub home: TeamCompData,
    pub away: TeamCompData,
    is_over: bool,
}

// Basics.
impl KnockoutPair {
    // Build the element.
    fn build(home: TeamCompData, away: TeamCompData) -> Self {
        let mut pair = Self::default();
        pair.home = home;
        pair.away = away;

        return pair;
    }

    // Get nice JSON for comp screen.
    fn get_comp_screen_json(&self) -> serde_json::Value {
        json!({
            "home": self.home.get_comp_screen_json_pair(),
            "away": self.away.get_comp_screen_json_pair()
        })
    }

    // Get the victor and the loser of the pair, or None if neither has won.
    fn get_winner_loser(&self, wins_required: u8) -> Option<[TeamCompData; 2]> {
        if self.home.get_wins() >= wins_required {
            return Some([self.home.clone(), self.away.clone()]);
        }
        if self.away.get_wins() >= wins_required {
            return Some([self.away.clone(), self.home.clone()]);
        }

        return None;
    }

    // Remove any upcoming games from these two teams.
    fn clean_up_games(&self, upcoming_games: &mut Vec<Game>) {
        upcoming_games.retain(|game| {
            game.home.team_id != self.home.team_id &&
            game.home.team_id != self.away.team_id &&
            game.away.team_id != self.home.team_id &&
            game.home.team_id != self.away.team_id
        });
    }

    // Update the teamdata for the pair.
    fn update_teamdata(&mut self, games: &[Game]) {
        for game in games.iter() {
            if self.home.team_id == game.home.team_id {
                self.home.update(&game.home, &game.away, game.has_overtime());
                self.away.update(&game.away, &game.home, game.has_overtime());
                break;
            }
            else if self.home.team_id == game.away.team_id {
                self.home.update(&game.away, &game.home, game.has_overtime());
                self.away.update(&game.home, &game.away, game.has_overtime());
                break;
            }
        }
    }
}