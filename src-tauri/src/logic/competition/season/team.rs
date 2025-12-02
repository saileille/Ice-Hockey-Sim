// Data for teams.

use ordinal::ToOrdinal;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::FromRow;

use crate::{logic::{competition::{Competition, round_robin::RoundRobin as RoundRobinFormat}, game::team::TeamGame, types::{Db, SeasonId, TeamId}}, packages::competition_screen::season::KnockoutTeamPackage};

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
    #[sqlx(rename = "ranking")]
    pub rank: u8,
    pub regular_wins: u8,
    pub ot_wins: u8,
    pub draws: u8,
    pub ot_losses: u8,
    pub regular_losses: u8,
    pub goals_scored: u16,
    pub goals_conceded: u16,
    games: u8,
    pub all_wins: u8,
    pub all_losses: u8,
    pub goal_difference: i16,
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

    // Get relevant information for a competition screen.
    pub async fn comp_screen_package(&self, db: &Db, comp: &Competition) -> serde_json::Value {
        json!({
            "id": self.team_id,
            "name": self.team_name(db).await,
            "seed": self.seed,
            "rank": self.rank.to_ordinal_string(),
            "games": self.games,
            "wins": self.regular_wins,
            "ot_wins": self.ot_wins,
            "draws": self.draws,
            "ot_losses": self.ot_losses,
            "losses": self.regular_losses,
            "total_wins": self.all_wins,
            "total_losses": self.all_losses,
            "goals_scored": self.goals_scored,
            "goals_conceded": self.goals_conceded,
            "goal_difference": self.goal_difference,
            "points": self.points(&comp.round_robin_format(db).await),
        })
    }

    // Get information for the competition screen tournament tree.
    /*pub async fn comp_screen_package_pair(&self, db: &Db) -> serde_json::Value {
        json!({
            "id": self.team_id,
            "name": self.team_name(db).await,
            "wins": self.all_wins,
            "seed": self.seed
        })
    }*/

    pub fn comp_screen_package_pair(&self) -> KnockoutTeamPackage {
        KnockoutTeamPackage::build(self)
    }
}

// Functional
impl TeamSeason {
    // Get points accumulated in a round robin stage.
    pub fn points(&self, rr_option: &Option<RoundRobinFormat>) -> u8 {
        if rr_option.is_none() { return 0; }
        let rr = rr_option.as_ref().unwrap();

        self.regular_wins * rr.points_for_win +
        self.ot_wins * rr.points_for_ot_win +
        self.draws * rr.points_for_draw +
        self.ot_losses * rr.points_for_ot_loss +
        self.regular_losses * rr.points_for_loss
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