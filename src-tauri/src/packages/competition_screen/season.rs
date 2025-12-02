use futures::TryStreamExt as _;
use ordinal::ToOrdinal as _;
use serde::Serialize;
use sqlx::{FromRow, Row, sqlite::SqliteRow};
use time::Date;

use crate::{logic::{competition::{self, round_robin::RoundRobin as RoundRobinFormat, season::{Season, knockout_round::KnockoutRound, team::TeamSeason}}, types::{CompetitionId, Db, SeasonId, TeamId}}, packages::competition_screen::game::GamePackage};



#[derive(Serialize)]
pub struct SeasonPackage {
    id: SeasonId,   // Simply for easier retrieval of other stuff.
    teams: Vec<TeamPackage>,
    knockout_rounds: Vec<KnockoutRoundPackage>,
    upcoming_games: Vec<GamePackage>,
    played_games: Vec<GamePackage>,
}

impl FromRow<'_, SqliteRow> for SeasonPackage {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        Ok(Self {
            id: row.try_get("id")?,
            teams: Vec::new(),
            knockout_rounds: Vec::new(),
            upcoming_games: Vec::new(),
            played_games: Vec::new(),
        })
    }
}

impl SeasonPackage {
    pub async fn finalise(&mut self, db: &Db, today: Date, o_rr_format: Option<&RoundRobinFormat>, comp_name: String, comp_type: competition::Type, comp_id: CompetitionId) {
        self.get_teams(db, o_rr_format).await;
        self.get_knockout_rounds(db, comp_name, comp_type, comp_id).await;
        self.get_games(db, today).await;
    }

    async fn get_teams(&mut self, db: &Db, o_rr_format: Option<&RoundRobinFormat>) {
        self.teams = sqlx::query_as(
            "SELECT team_id, Team.full_name, seed, ranking, regular_wins,
            ot_wins, draws, ot_losses, regular_losses, goals_scored, goals_conceded,
            all_wins, all_losses, games, goal_difference
            FROM TeamSeason

            INNER JOIN Team ON Team.id = TeamSeason.team_id

            WHERE season_id = $1"
        ).bind(self.id)
        .fetch_all(db).await.unwrap();

        if o_rr_format.is_none() { return; }
        let rr_format = o_rr_format.unwrap();
        for team in self.teams.iter_mut() {
            team.count_points(&rr_format);
        }
    }

    async fn get_knockout_rounds(&mut self, db: &Db, comp_name: String, comp_type: competition::Type, comp_id: CompetitionId) {
        if comp_type == competition::Type::KnockoutRound {
            let knockout_round = Season::knockout_round_from_id(db, self.id).await;
            self.knockout_rounds = vec![KnockoutRoundPackage::build(&knockout_round, comp_name)];
        }

        else if comp_type == competition::Type::Tournament {
            let knockout_rounds = Season::child_knockout_rounds_from_id(db, comp_id).await;
            self.knockout_rounds = knockout_rounds.into_iter().map(|a| KnockoutRoundPackage::build(&a, comp_name.clone())).collect();
        }
    }

    async fn get_games(&mut self, db: &Db, today: Date) {
        let select = GamePackage::select_query();
        let query = format!("
            {select}
            WHERE Home.season_id = $1
            ORDER BY unixepoch(date) ASC
        ");

        let mut rows = sqlx::query(query.as_str())
        .bind(self.id)
        .fetch(db);

        while let Some(row) = rows.try_next().await.unwrap() {
            let game = GamePackage::from_row(&row).unwrap();

            match game.date < today {
                true => self.played_games.push(game),
                false => self.upcoming_games.push(game)
            };
        }

        // Reversing played games so that the most recent is first.
        self.played_games.reverse();
    }
}

#[derive(Serialize)]
pub struct TeamPackage {
    id: TeamId,
    name: String,
    seed: u8,
    rank: String,
    games: u8,
    regular_wins: u8,
    ot_wins: u8,
    draws: u8,
    ot_losses: u8,
    regular_losses: u8,
    all_wins: u8,
    all_losses: u8,
    goals_scored: u16,
    goals_conceded: u16,
    goal_difference: i16,
    points: u8,
}

impl FromRow<'_, SqliteRow> for TeamPackage {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        let rank_u8: u8 = row.try_get("ranking")?;
        Ok(Self {
            id: row.try_get("team_id")?,
            name: row.try_get("full_name")?,
            seed: row.try_get("seed")?,
            rank: rank_u8.to_ordinal_string(),
            games: row.try_get("games")?,
            regular_wins: row.try_get("regular_wins")?,
            ot_wins: row.try_get("ot_wins")?,
            draws: row.try_get("draws")?,
            ot_losses: row.try_get("ot_losses")?,
            regular_losses: row.try_get("regular_losses")?,
            all_wins: row.try_get("all_wins")?,
            all_losses: row.try_get("all_losses")?,
            goals_scored: row.try_get("goals_scored")?,
            goals_conceded: row.try_get("goals_conceded")?,
            goal_difference: row.try_get("goal_difference")?,
            points: u8::default(),
        })
    }
}

impl TeamPackage {
    pub fn count_points(&mut self, rr_format: &RoundRobinFormat) {
        self.points =
            self.regular_wins * rr_format.points_for_win +
            self.ot_wins * rr_format.points_for_ot_win +
            self.draws * rr_format.points_for_draw +
            self.ot_losses * rr_format.points_for_ot_loss +
            self.regular_losses * rr_format.points_for_loss
        ;
    }
}

#[derive(Serialize)]
pub struct KnockoutTeamPackage {
    id: TeamId,
    name: String,
    wins: u8,
    seed: u8,
}

impl KnockoutTeamPackage {
    pub fn custom_from_row(row: &SqliteRow, home_away: &str) -> sqlx::Result<Self> {
        Ok(Self {
            id: row.try_get(format!("{home_away}_id").as_str())?,
            name: row.try_get(format!("{home_away}_full_name").as_str())?,
            seed: row.try_get(format!("{home_away}_seed").as_str())?,
            wins: row.try_get(format!("{home_away}_wins").as_str())?,
        })
    }

    // Temporary thing, do not use.
    pub fn build(team: &TeamSeason) -> Self {
        Self {
            id: team.team_id,
            name: String::new(),
            wins: team.all_wins,
            seed: team.seed,
        }
    }
}

#[derive(Serialize)]
pub struct KnockoutRoundPackage {
    name: String,
    pub pairs: Vec<KnockoutPairPackage>,
}

impl KnockoutRoundPackage {
    pub fn build(knockout_round: &KnockoutRound, comp_name: String) -> Self {
        Self {
            name: comp_name,
            pairs: knockout_round.pairs.iter().map(|pair| pair.comp_screen_package()).collect(),
        }
    }
}

#[derive(Serialize)]
pub struct KnockoutPairPackage {
    pub home: KnockoutTeamPackage,
    pub away: KnockoutTeamPackage,
}

impl FromRow<'_, SqliteRow> for KnockoutPairPackage {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        Ok(Self {
            home: KnockoutTeamPackage::custom_from_row(row, "home")?,
            away: KnockoutTeamPackage::custom_from_row(row, "away")?,
        })
    }
}