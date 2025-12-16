pub mod editor;

use serde::Serialize;
use sqlx::FromRow;

use crate::{db::AppData, logic::CompetitionId};

#[derive(FromRow)]
#[derive(Serialize)]
pub struct CompetitionSelect {
    id: CompetitionId,
    name: String,
}

impl CompetitionSelect {
    pub async fn all(data: &AppData) -> Vec<Self> {
        let comps: Vec<Self> = sqlx::query_as(
            "SELECT * FROM CompetitionSelect"
        ).fetch_all(&data.db_info.pool).await.unwrap();

        return comps;
    }
}