mod country;

use serde::Serialize;
use sqlx::{FromRow, Row, sqlite::SqliteRow};
use time::Date;

use crate::{logic::{person::attribute::PersonAttribute, time::years_between, types::{Db, PersonId}}, packages::player_search_screen::country::CountryPackage};

#[derive(Serialize)]
pub struct PlayerPackage {
    person: PersonPackage,
    position: String,
    ability: u8,
}

/*impl FromRow<'_, SqliteRow> for PlayerPackage {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        Ok(Self {
            person: PersonPackage::from_row(row)?,
        })
    }
}*/

impl PlayerPackage {
    // Get all free agents.
    fn custom_from_row(row: &SqliteRow, today: Date) -> sqlx::Result<Self> {
        Ok(Self {
            person: PersonPackage::custom_from_row(row, today)?,
            position: row.try_get("position_name")?,
            ability: PersonAttribute::display(row.try_get("ability")?),
        })
    }

    // Get all free agents as a player search.
    pub async fn free_agents(db: &Db, today: Date) -> Vec<Self> {
        let mut players = sqlx::query(
            "SELECT ability, abbreviation AS position_name,
            Person.id, full_name, birthday,
            Country.country_name, flag_path,
            (
                SELECT COUNT(*) FROM Contract
                WHERE person_id = Person.id
                AND is_signed = FALSE
            ) AS no_of_offers
            FROM Player

            INNER JOIN Position ON Position.id = position_id
            INNER JOIN Person ON Person.id = person_id
            INNER JOIN Country ON country_id = Country.id

            WHERE Person.id NOT IN (
                SELECT person_id FROM Contract
                WHERE is_signed = TRUE
            ) AND Person.is_active = TRUE
            ORDER BY position_id ASC, surname ASC, forename ASC"
        ).map(|row| PlayerPackage::custom_from_row(&row, today).unwrap())
        .fetch_all(db).await.unwrap();

        players.sort_by(|a, b| b.ability.cmp(&a.ability));

        return players;
    }
}

#[derive(Serialize)]
struct PersonPackage {
    id: PersonId,
    full_name: String,
    country: CountryPackage,
    age: i8,
    no_of_offers: u8,
}

/*impl FromRow<'_, SqliteRow> for PersonPackage {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        Ok(Self {
            id: row.try_get("id")?,
            full_name: row.try_get("full_name")?,
            country: CountryPackage::from_row(row)?,
            birthday: row.try_get("birthday")?,
            age: i8::default(),
            no_of_offers: row.try_get("no_of_offers")?,
        })
    }
}*/

impl PersonPackage {
    fn custom_from_row(row: &SqliteRow, today: Date) -> sqlx::Result<Self> {
        let birthday = row.try_get("birthday")?;
        Ok(Self {
            id: row.try_get("id")?,
            full_name: row.try_get("full_name")?,
            country: CountryPackage::from_row(row)?,
            age: years_between(birthday, today),
            no_of_offers: row.try_get("no_of_offers")?,
        })
    }
}