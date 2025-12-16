use serde::Serialize;
use serde_json::json;
use sqlx::{FromRow, Row, sqlite::SqliteRow};

use crate::logic::{CompetitionId, Db, competition::RankCriteria, time::AnnualWindow};

#[derive(Default)]
#[derive(Serialize)]
enum CompType {
    #[default]
    Container,
    RoundRobin {
        rounds: u8,
        extra_matches: u8,
        points_for_win: u8,
        points_for_ot_win: u8,
        points_for_draw: u8,
        points_for_ot_loss: u8,
        points_for_loss: u8,
    },
    KnockoutRound {
        wins_required: u8,
    },
    Tournament,
}

// The editor view of a competition.
#[derive(Default)]
#[derive(Serialize)]
pub struct Competition {
    id: CompetitionId,
    comp_name: String,
    season_window: AnnualWindow,
    min_no_of_teams: u8,
    rank_criteria: Vec<RankCriteria>,
    comp_type: CompType,
    parent_id: CompetitionId,

    // GameRules
    periods: u8,
    period_length: u16,
    overtime_length: u16,
    continuous_overtime: bool,
}

impl FromRow<'_, SqliteRow> for Competition {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        let comp_type: String = row.try_get("comp_type")?;
        Ok(Self {
            id: row.try_get("id")?,
            comp_name: row.try_get("comp_name")?,
            season_window: row.try_get("season_window")?,
            min_no_of_teams: row.try_get("min_no_of_teams")?,
            rank_criteria: serde_json::from_value(row.try_get("rank_criteria")?).unwrap(),
            comp_type: CompType::Container, // Placeholder.
            parent_id: match row.try_get("parent_id")? {
                Some(v) => v,
                None => 0
            },

            ..Default::default()
            // Needs much more...
        })
    }
}

impl Competition {
    // Get a competition for the editor screen based on competition ID.
    pub async fn from_id(id: CompetitionId, db: &Db) -> Self {
        return sqlx::query_as(
            "SELECT * FROM CompetitionEditorScreen
            WHERE id = $1"
        ).bind(id)
        .fetch_one(db).await.unwrap()
    }

    // Get the default instance of the object. Used when creating a new competition.
    pub async fn create_default(db: &Db) -> Self {
        let mut comp = Self {
            comp_name: "new competition".to_string(),
            ..Default::default()
        };

        comp.id = sqlx::query_scalar(
            "INSERT INTO Competition
            (comp_name, season_window, min_no_of_teams, rank_criteria, comp_type)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id"
        ).bind(comp.comp_name.as_str())
        .bind(&comp.season_window)
        .bind(comp.min_no_of_teams)
        .bind(json!(comp.rank_criteria))
        .bind(json!(comp.comp_type))
        .fetch_one(db).await.unwrap();

        return comp;
    }
}