use sqlx::{FromRow, Row, sqlite::SqliteRow};

use crate::logic::{person::{Person, attribute::{AttributeId, PersonAttribute}, player::{Player, position::PositionId}}, types::{Db, PersonId, TeamId}};

impl FromRow<'_, SqliteRow> for Player {
    fn from_row(row: &SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            person: Person::from_row(row)?,
            ability: PersonAttribute::build(AttributeId::General, row.try_get("ability")?),
            position_id: row.try_get("position_id")?,
        })
    }
}

impl Player {
    // Remove all contract offers that the player has.
    pub async fn reject_contracts(&self, db: &Db) {
        sqlx::query(
            "DELETE FROM Contract
            WHERE person_id = $1 AND is_signed = FALSE"
        ).bind(self.person.id)
        .execute(db).await.unwrap();
    }

    // Get a player from the database.
    pub async fn fetch_from_db(db: &Db, id: PersonId) -> Option<Self> {
        sqlx::query_as(
            "SELECT Person.*, Player.ability, Player.position_id FROM Person
            INNER JOIN Player ON Person.id = Player.person_id
            WHERE Person.id = $1"
        ).bind(id)
        .fetch_optional(db).await.unwrap()
    }

    // Get all active players.
    pub async fn fetch_active(db: &Db) -> Vec<Self> {
        sqlx::query_as(
            "SELECT Person.*, Player.ability, Player.position_id FROM Person
            INNER JOIN Player ON Person.id = Player.person_id
            WHERE Person.is_active = TRUE"
        ).fetch_all(db).await.unwrap()
    }

    // Save to database.
    pub async fn save(&self, db: &Db) {
        sqlx::query(
            "INSERT INTO Player
            (person_id, ability, position_id)
            VALUES ($1, $2, $3)"
        ).bind(self.person.id)
        .bind(self.ability.value)
        .bind(self.position_id)
        .execute(db).await.unwrap();
    }

    // Get a clone of the player's position abbreviation.
    pub async fn position_abbr(&self, db: &Db) -> String {
        sqlx::query_scalar(
            "SELECT abbreviation FROM Position
            WHERE id = $1"
        ).bind(self.position_id)
        .fetch_one(db).await.unwrap()
    }

    // Get all free agents from the database with given positions, which the given team has not approached yet.
    pub async fn free_agents_for_team(db: &Db, positions: Vec<PositionId>, team_id: TeamId) -> Vec<Self> {
        let mut position_query = String::new();
        for position in positions {
            let pos_string = match position_query.is_empty() {
                true => format!("{}", position as u8),
                false => format!(", {}", position as u8)
            };
            position_query.push_str(pos_string.as_str())
        }

        sqlx::query_as(format!("
            SELECT Person.*, Player.ability, Player.position_id FROM Person
            INNER JOIN Player ON Player.person_id = Person.id
            WHERE Person.is_active = TRUE AND PLAYER.position_id IN ({position_query}) AND
            Person.id NOT IN (
                SELECT person_id FROM Contract
                WHERE is_signed = TRUE
                OR team_id = $1
            )
        ").as_str())
        .bind(team_id)
        .fetch_all(db).await.unwrap()
    }

    // Get all free agents.
    pub async fn free_agents(db: &Db) -> Vec<Self> {
        sqlx::query_as(
            "SELECT Person.*, Player.ability, Player.position_id FROM Person
            INNER JOIN Player ON Player.person_id = Person.id
            WHERE Person.id NOT IN (
                SELECT person_id FROM Contract
                WHERE is_signed = TRUE
            ) AND Person.is_active = TRUE
            ORDER BY position_id ASC, surname ASC, forename ASC"
        )
        .fetch_all(db).await.unwrap()
    }

    pub async fn update_ability(&self, db: &Db) {
        sqlx::query(
            "UPDATE Player SET ability = $1
            WHERE person_id = $2"
        ).bind(self.ability.value)
        .bind(self.person.id)
        .execute(db).await.unwrap();
    }
}