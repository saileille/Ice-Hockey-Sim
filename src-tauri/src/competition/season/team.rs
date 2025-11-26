// Data for teams.

use std::num::NonZero;

use ordinal::ToOrdinal;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::FromRow;

use crate::{competition::{Competition, format}, match_event::team::TeamGame, types::{Db, SeasonId, TeamId, convert}};

#[derive(Debug, Serialize, Deserialize)]
#[derive(PartialEq)]
#[derive(Default, Clone)]
#[derive(FromRow)]
pub struct TeamSeason {
    pub team_id: TeamId,
    pub season_id: SeasonId,

    // Seed is mostly used in knockouts, but can be used for tie-breakers in round-robin as well.
    // The lower the value, the better the seed is.
    // 0 can theoretically be used, but for clarity, maybe use it only when every team's seed is 0?
    pub seed: u8,
    rank: u8,
    pub regular_wins: u8,
    pub ot_wins: u8,
    pub draws: u8,
    pub ot_losses: u8,
    pub regular_losses: u8,
    pub goals_scored: u16,
    pub goals_conceded: u16,
}

// Basics.
impl TeamSeason {
    pub fn build(team_id: TeamId, seed: u8) -> Self {
        Self {
            team_id: team_id,
            seed: seed,
            ..Default::default()
        }
    }

    pub async fn save(&self, db: &Db) {
        sqlx::query(
            "INSERT INTO TeamSeason
            (team_id, season_id, seed, rank, regular_wins, ot_wins, draws, ot_losses, regular_losses, goals_scored, goals_conceded)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"
        ).bind(NonZero::new(self.team_id).unwrap())
        .bind(NonZero::new(self.season_id).unwrap())
        .bind(self.seed)
        .bind(self.rank)
        .bind(self.regular_wins)
        .bind(self.ot_wins)
        .bind(self.draws)
        .bind(self.ot_losses)
        .bind(self.regular_losses)
        .bind(self.goals_scored)
        .bind(self.goals_conceded)
        .execute(db).await.unwrap();
    }

    async fn team_name(&self, db: &Db) -> String {
        sqlx::query_scalar(
            "SELECT full_name FROM Team WHERE id = $1"
        ).bind(self.team_id)
        .fetch_one(db).await.unwrap()
    }

    // Get relevant information for a competition screen.
    pub async fn comp_screen_package(&self, db: &Db, comp: &Competition) -> serde_json::Value {
        json!({
            "id": self.team_id,
            "name": self.team_name(db).await,
            "seed": self.seed,
            "rank": self.rank.to_ordinal_string(),
            "games": self.games(),
            "wins": self.regular_wins,
            "ot_wins": self.ot_wins,
            "draws": self.draws,
            "ot_losses": self.ot_losses,
            "losses": self.regular_losses,
            "total_wins": self.all_wins(),
            "total_losses": self.all_losses(),
            "goals_scored": self.goals_scored,
            "goals_conceded": self.goals_conceded,
            "goal_difference": self.goal_difference(),
            "points": self.points(&comp.round_robin_format()),
        })
    }

    // Get information for the competition screen tournament tree.
    pub async fn comp_screen_package_pair(&self, db: &Db) -> serde_json::Value {
        json!({
            "id": self.team_id,
            "name": self.team_name(db).await,
            "wins": self.all_wins(),
            "seed": self.seed
        })
    }
}

// Functional
impl TeamSeason {
    pub fn games(&self) -> u8 {
        self.all_wins() + self.all_losses() + self.draws
    }

    pub fn all_wins(&self) -> u8 {
        self.regular_wins + self.ot_wins
    }

    pub fn all_losses(&self) -> u8 {
        self.regular_losses + self.ot_losses
    }

    // Get points accumulated in a round robin stage.
    pub fn points(&self, rr_option: &Option<format::round_robin::RoundRobin>) -> u8 {
        if rr_option.is_none() { return 0; }
        let rr = rr_option.as_ref().unwrap();

        self.regular_wins * rr.points_for_win +
        self.ot_wins * rr.points_for_ot_win +
        self.draws * rr.points_for_draw +
        self.ot_losses * rr.points_for_ot_loss +
        self.regular_losses * rr.points_for_loss
    }

    pub fn goal_difference(&self) -> i16 {
        let gf = convert::int::<u16, i16>(self.goals_scored);
        let ga = convert::int::<u16, i16>(self.goals_conceded);
        return gf - ga;
    }

    pub fn update(&mut self, game_data: &TeamSeason) {
        self.regular_wins += game_data.regular_wins;
        self.ot_wins += game_data.ot_wins;
        self.draws += game_data.draws;
        self.ot_losses += game_data.ot_losses;
        self.regular_losses += game_data.regular_losses;
        self.goals_scored += game_data.goals_scored;
        self.goals_conceded += game_data.goals_conceded;
    }

    // Update the team data after a match.
    pub async fn update_and_save(&mut self, db: &Db, game_data: &TeamSeason) {
        self.update(game_data);

        sqlx::query(
            "UPDATE TeamSeason SET
            regular_wins = $1, ot_wins = $2, draws = $3,
            ot_losses = $4, regular_losses = $5,
            goals_scored = $6, goals_conceded = $7
            WHERE team_id = $8 AND season_id = $9"
        ).bind(self.regular_wins)
        .bind(self.ot_wins)
        .bind(self.draws)
        .bind(self.ot_losses)
        .bind(self.regular_losses)
        .bind(self.goals_scored)
        .bind(self.goals_conceded)
        .bind(self.team_id)
        .bind(self.season_id)
        .execute(db).await.unwrap();
    }

    // Get TeamSeason objects from a game.
    pub fn season_data_from_game(home: &TeamGame, away: &TeamGame, overtime: bool) -> (TeamSeason, TeamSeason) {
        let mut home_data = TeamSeason::build(home.team_id, 0);
        let mut away_data = TeamSeason::build(away.team_id, 0);

        let home_goals = home.goals();
        let away_goals = away.goals();

        if home_goals > away_goals {
            if !overtime {
                home_data.regular_wins = 1;
                away_data.regular_losses = 1;
            }
            else {
                home_data.ot_wins = 1;
                away_data.ot_losses = 1;
            }
        }
        else if away_goals > home_goals {
            if !overtime {
                away_data.regular_wins = 1;
                home_data.regular_losses = 1;
            }
            else {
                away_data.ot_wins = 1;
                home_data.ot_losses = 1;
            }
        }
        else {
            home_data.draws = 1;
            away_data.draws = 1;
        }

        home_data.goals_scored = home_goals;
        away_data.goals_scored = away_goals;
        home_data.goals_conceded = away_goals;
        away_data.goals_conceded = home_goals;

        return (home_data, away_data);
    }
}