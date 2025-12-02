mod game;
pub mod season;

use serde::Serialize;
use sqlx::{FromRow, Row, sqlite::SqliteRow};
use time::Date;

use crate::{logic::{competition::{self, Type, round_robin::RoundRobin as RoundRobinFormat}, types::{CompetitionId, Db}}, packages::competition_screen::season::SeasonPackage};

// For more optimised queries, I suppose.
#[derive(Serialize)]
pub struct CompetitionPackage {
    id: CompetitionId,
    parent_id: CompetitionId,
    name: String,
    full_name: String,
    season: SeasonPackage,
    comp_nav: Vec<Vec<(CompetitionId, String)>>,
    competition_type: competition::Type,
    round_robin_format: Option<RoundRobinFormat>,
}

impl FromRow<'_, SqliteRow> for CompetitionPackage {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        let competition_type = row.try_get("comp_type")?;
        Ok(Self {
            id: row.try_get("competition_id")?,
            parent_id: match row.try_get("parent_id")? {
                Some(id) => id,
                None => 0,
            },
            name: row.try_get("comp_name")?,
            full_name: String::new(),
            season: SeasonPackage::from_row(row)?,
            comp_nav: Vec::new(),
            competition_type,
            round_robin_format: match competition_type {
                Type::RoundRobin => Some(RoundRobinFormat::from_row(row)?),
                _ => None,
            },
        })
    }
}

impl CompetitionPackage {
    pub async fn build(db: &Db, today: Date, comp_id: CompetitionId) -> Self {
        let mut package: Self = sqlx::query_as(
            "SELECT
            Competition.id AS competition_id, parent_id, comp_name, comp_type, Season.id,
            RoundRobinFormat.*
            FROM Competition

            INNER JOIN Season ON Season.comp_id = Competition.id
            LEFT JOIN RoundRobinFormat ON RoundRobinFormat.comp_id = Competition.id

            WHERE Competition.id = $1
            GROUP BY Season.comp_id
            ORDER BY Season.id DESC"
        ).bind(comp_id)
        .fetch_one(db).await.unwrap();

        package.get_full_name(db).await;
        package.create_comp_nav(db).await;

        package.season.finalise(db, today, package.round_robin_format.as_ref(), package.name.to_string(), package.competition_type, comp_id).await;
        return package;
    }

    async fn get_full_name(&mut self, db: &Db) {
        self.full_name = self.name.clone();
        let mut o_parent = Self::parent_id_and_name(db, self.parent_id).await;
        while o_parent.is_some() {
            let parent = o_parent.unwrap();
            self.full_name = format!("{} {}", parent.1, self.full_name);
            o_parent = Self::parent_id_and_name(db, parent.0).await;
        }
    }

    // Get the parent of this competition.
    pub async fn parent(&self, db: &Db) -> Option<Self> {
        sqlx::query_as(
            "SELECT Competition.id AS competition_id, parent_id, comp_name, comp_type, Season.id FROM Competition
            INNER JOIN Season ON Season.comp_id = Competition.id
            WHERE Competition.id = $1
            GROUP BY Season.comp_id
            ORDER BY Season.id DESC"
        ).bind(self.parent_id)
        .fetch_optional(db).await.unwrap()
    }

    // Get the parent of this competition.
    pub async fn parent_id_and_name(db: &Db, comp_id: CompetitionId) -> Option<(CompetitionId, String)> {
        sqlx::query_as(
            "SELECT parent_id, comp_name FROM Competition
            WHERE Competition.id = $1"
        ).bind(comp_id)
        .fetch_optional(db).await.unwrap()
    }

    // Create a full competition tree.
    async fn create_comp_nav(&mut self, db: &Db) {
        let mut comp_nav = Vec::new();

        let children = self.child_nav_package(db, "Overview").await;
        if children.len() > 1 { comp_nav.push(children); }

        let siblings = self.sibling_nav_package(db).await;
        if siblings.len() > 1 { comp_nav.push(siblings); }

        self.parent_nav_package(db, &mut comp_nav).await;

        // We need to reverse so we get the competition hierarchy from highest to lowest.
        comp_nav.reverse();
        self.comp_nav = comp_nav;
    }

    // Get the IDs and names of all parent competitions and their siblings.
    async fn parent_nav_package(&self, db: &Db, select_data: &mut Vec<Vec<(CompetitionId, String)>>) {
        let mut o_parent = self.parent(db).await;
        while o_parent.is_some() {
            let parent = o_parent.as_ref().unwrap();

            let uncles = parent.sibling_nav_package(db).await;
            if uncles.len() > 1 { select_data.push(uncles); }

            o_parent = parent.parent(db).await;
        }
    }

    // Get the IDs of sibling competitions, including this one.
    async fn sibling_nav_package(&self, db: &Db) -> Vec<(CompetitionId, String)> {
        let mut siblings = match self.parent(db).await {
            Some(parent) => parent.child_nav_package(db, parent.name.as_str()).await,
            _ => vec![(self.id, self.name.clone())]
        };

        siblings.sort_by(|(a, _), (b, _)| a.cmp(&b));

        // Replace this competition's ID with 0, to keep track of which competition is selected.
        for comp in siblings.iter_mut() {
            if comp.0 == self.id {
                comp.0 = 0;
                break;
            }
        }

        return siblings;
    }

    // Get the IDs and names of child competitions.
    async fn child_nav_package(&self, db: &Db, self_name: &str) -> Vec<(CompetitionId, String)> {
        let mut children = self.child_navs(db).await;

        // Adding this competition so selecting a child becomes possible.
        children.insert(0, (self.id, self_name.to_string()));
        return children;
    }

    // Get the ID and name of the children.
    async fn child_navs(&self, db: &Db) -> Vec<(CompetitionId, String)> {
        sqlx::query_as(
            "SELECT id, comp_name FROM Competition
            WHERE parent_id = $1
            ORDER BY id ASC"
        ).bind(self.id)
        .fetch_all(db).await.unwrap()
    }
}