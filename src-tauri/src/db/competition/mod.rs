mod season;

mod comp_connection;
mod knockout_round;
mod round_robin;

use serde_json::json;
use sqlx::{FromRow, Row, sqlite::{SqliteQueryResult, SqliteRow}};

use crate::logic::{competition::{Competition, comp_connection::CompConnection, knockout_round::KnockoutRound as KnockoutRoundFormat, round_robin::RoundRobin as RoundRobinFormat, season::{Season, team::TeamSeason}}, game, types::{CompetitionId, Db, SeasonId, TeamId}};

impl FromRow<'_, SqliteRow> for Competition {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        Ok(Self {
            id: row.try_get("id")?,
            name: row.try_get("comp_name")?,
            season_window: row.try_get("season_window")?,
            min_no_of_teams: row.try_get("min_no_of_teams")?,
            rank_criteria: serde_json::from_value(row.try_get("rank_criteria")?).unwrap(),
            comp_type: row.try_get("comp_type")?,
            parent_id: match row.try_get("parent_id")? {
                Some(v) => v,
                None => 0
            },
            rr_id: row.try_get("rr_format_id")?,
            kr_id: row.try_get("kr_format_id")?,
            game_rules_id: row.try_get("game_rules_id")?,
        })
    }
}

// Static read queries
impl Competition {
    // Fetch a competition from the database, knowing that it exists.
    pub async fn fetch_from_db(db: &Db, id: CompetitionId) -> Self {
        sqlx::query_as(
            "SELECT * FROM Competition WHERE id = $1"
        ).bind(id)
        .fetch_one(db).await.unwrap()
    }

    // Get all competitions that do not have a parent.
    pub async fn fetch_parents(db: &Db) -> Vec<Self> {
        sqlx::query_as(
            "SELECT * FROM Competition
            WHERE parent_id IS NULL"
        ).fetch_all(db).await.unwrap()
    }

    // Get the ID and name of all parent competitions.
    pub async fn fetch_parent_id_and_name(db: &Db) -> Vec<(CompetitionId, String)> {
        sqlx::query_as(
            "SELECT id, comp_name FROM Competition
            WHERE parent_id IS NULL
            ORDER BY id ASC"
        ).fetch_all(db).await.unwrap()
    }

    // Fetch competitions that have a format (i.e. games).
    pub async fn fetch_comps_with_games(db: &Db) -> Vec<Self> {
        sqlx::query_as(
            // Using 'IN' instead of '!=' to make use of the index.
            "SELECT * FROM Competition WHERE comp_type IN ('RoundRobin', 'KnockoutRound', 'Tournament')"
        ).fetch_all(db).await.unwrap()
    }
}

// Database write queries.
impl Competition {
    // Combining the two async functions to not nest them.
    pub async fn save(&mut self, db: &Db) {
        self.id = sqlx::query_scalar(
            "INSERT INTO Competition
            (comp_name, season_window, min_no_of_teams, rank_criteria, comp_type, rr_format_id, kr_format_id, game_rules_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id"
        ).bind(self.name.as_str())
        .bind(&self.season_window)
        .bind(self.min_no_of_teams)
        .bind(json!(self.rank_criteria))
        .bind(self.comp_type)
        .bind(self.rr_id)
        .bind(self.kr_id)
        .bind(self.game_rules_id)
        .fetch_one(db).await.unwrap();
    }

    pub async fn save_with_parent_id(&mut self, db: &Db) {
        self.id = sqlx::query_scalar(
            "INSERT INTO Competition
            (comp_name, season_window, min_no_of_teams, rank_criteria, comp_type, parent_id, rr_format_id, kr_format_id, game_rules_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id"
        ).bind(self.name.as_str())
        .bind(&self.season_window)
        .bind(self.min_no_of_teams)
        .bind(json!(self.rank_criteria))
        .bind(self.comp_type)
        .bind(self.parent_id)
        .bind(self.rr_id)
        .bind(self.kr_id)
        .bind(self.game_rules_id)
        .fetch_one(db).await.unwrap();
    }

    pub async fn save_parent_id(&self, db: &Db, parent_id: CompetitionId) -> SqliteQueryResult {
        sqlx::query(
            "UPDATE Competition SET parent_id = $1
            WHERE id = $2"
        ).bind(parent_id)
        .bind(self.id)
        .execute(db).await.unwrap()
    }

    pub async fn save_type(&self, db: &Db) -> SqliteQueryResult {
        sqlx::query(
            "UPDATE Competition SET comp_type = $1 WHERE id = $2"
        ).bind(self.comp_type)
        .bind(self.id)
        .execute(db).await.unwrap()
    }
}

// Database read queries.
impl Competition {
    pub async fn connections_to(&self, db: &Db) -> Vec<CompConnection> {
        sqlx::query_as(
            "SELECT * FROM CompConnection WHERE origin_id = $1"
        ).bind(self.id)
        .fetch_all(db).await.unwrap()
    }

    async fn match_rules(&self, db: &Db) -> game::Rules {
        sqlx::query_as(
            "SELECT * FROM GameRules WHERE id = $1"
        ).bind(self.game_rules_id)
        .fetch_one(db).await.unwrap()
    }

    // Get the round robin format, if competition has one.
    pub async fn round_robin_format(&self, db: &Db) -> Option<RoundRobinFormat> {
        sqlx::query_as(
            "SELECT * FROM RoundRobinFormat
            WHERE id = $1"
        ).bind(self.rr_id)
        .fetch_optional(db).await.unwrap()
    }

    pub async fn knockout_round_format(&self, db: &Db) -> Option<KnockoutRoundFormat> {
        sqlx::query_as(
            "SELECT * FROM KnockoutRoundFormat
            WHERE id = $1"
        ).bind(self.rr_id)
        .fetch_optional(db).await.unwrap()
    }

    pub async fn knockout_round_maximum_games(&self, db: &Db) -> Option<u8> {
        sqlx::query_scalar(
            "SELECT maximum_games FROM KnockoutRoundFormat
            WHERE id = $1"
        ).bind(self.kr_id)
        .fetch_optional(db).await.unwrap()
    }

    pub async fn child_ids(&self, db: &Db) -> Vec<CompetitionId> {
        sqlx::query_scalar(
            "SELECT id FROM Competition WHERE parent_id = $1"
        ).bind(self.id)
        .fetch_all(db).await.unwrap()
    }

    // Return the child competitions in the order that they are scheduled.
    // E.g. regular season should come before playoffs.
    pub async fn children(&self, db: &Db) -> Vec<Self> {
        sqlx::query_as(
            "SELECT * FROM Competition
            WHERE parent_id = $1
            ORDER BY id ASC"
        ).bind(self.id)
        .fetch_all(db).await.unwrap()
    }

    // Get the ID and name of the children.
    pub async fn child_navs(&self, db: &Db) -> Vec<(CompetitionId, String)> {
        sqlx::query_as(
            "SELECT id, comp_name FROM Competition
            WHERE parent_id = $1
            ORDER BY id ASC"
        ).bind(self.id)
        .fetch_all(db).await.unwrap()
    }

    pub async fn child_current_seasons(&self, db: &Db) -> Vec<Season> {
        sqlx::query_as(
            "SELECT Season.* FROM Season
            INNER JOIN Competition ON Competition.id = Season.comp_id
            WHERE Competition.parent_id = $1
            GROUP BY Season.comp_id
            ORDER BY Competition.id ASC, Season.id DESC"
        ).bind(self.id)
        .fetch_all(db).await.unwrap()
    }

    // Get the parent of this competition.
    pub async fn parent(&self, db: &Db) -> Option<Self> {
        sqlx::query_as(
            "SELECT * FROM Competition WHERE id = $1"
        ).bind(self.parent_id)
        .fetch_optional(db).await.unwrap()
    }

    // Get the current season of the competition.
    pub async fn current_season(&self, db: &Db) -> Season {
        sqlx::query_as(
            "SELECT * FROM Season
            WHERE comp_id = $1
            ORDER BY id DESC
            LIMIT 1"
        ).bind(self.id)
        .fetch_one(db).await.unwrap()
    }

    // Get the current season ID.
    pub async fn current_season_id(&self, db: &Db) -> SeasonId {
        sqlx::query_scalar(
            "SELECT id FROM Season
            WHERE comp_id = $1
            ORDER BY id DESC
            LIMIT 1"
        ).bind(self.id)
        .fetch_one(db).await.unwrap()
    }

    // Get the teams in the competition's current season.
    pub async fn current_season_teamdata(&self, db: &Db) -> Vec<TeamSeason> {
        sqlx::query_as(
            "SELECT * FROM TeamSeason
            WHERE season_id = (
                SELECT id FROM Season
                WHERE comp_id = $1
                ORDER BY id DESC
                LIMIT 1
            )
            ORDER BY ranking ASC"
        ).bind(self.id)
        .fetch_all(db).await.unwrap()
    }

    // Get the IDs and names of the current season's teams, based on competition ID.
    pub async fn current_season_team_select_data_by_id(db: &Db, id: CompetitionId) -> Vec<(TeamId, String)> {
        sqlx::query_as(
            "SELECT Team.id, Team.full_name FROM Team
            INNER JOIN TeamSeason ON
            TeamSeason.team_id = Team.id
            WHERE TeamSeason.season_id = (
                SELECT id FROM Season
                WHERE comp_id = $1
                ORDER BY id DESC
                LIMIT 1
            )
            ORDER BY Team.full_name ASC"
        ).bind(id)
        .fetch_all(db).await.unwrap()
    }
}