pub mod player;
pub mod manager;
pub mod attribute;


use rand::{self};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::FromRow;
use time::{Date, Duration};

use crate::{
    competition::Competition, country::Country, database, team::Team, time::{date_to_string, get_years_between, years_to_days}, types::{CountryId, Db, PersonId, TeamId}
};

#[derive(Eq, Hash)]
#[derive(Debug)]
#[derive(Default, Copy, Clone, PartialEq)]
#[derive(sqlx::Type, Serialize, Deserialize)]
pub enum Gender {
    #[default] Null,
    Male,
    Female,
}

#[derive(Debug, Clone)]
#[derive(FromRow)]
pub struct Person {
    pub id: PersonId,
    pub forename: String,
    pub surname: String,
    gender: Gender,
    country_id: CountryId,
    birthday: Date,
    pub is_active: bool,
}

impl Default for Person {
    fn default() -> Self {
        Self {
            id: PersonId::default(),
            forename: String::default(),
            surname: String::default(),
            gender: Gender::default(),
            country_id: CountryId::default(),
            birthday: Date::MIN,
            is_active: bool::default(),
        }
    }
}

// Basics.
impl Person {
    // Get the next ID to use.
    async fn next_id(db: &Db) -> PersonId {
        let max: Option<PersonId> = sqlx::query_scalar("SELECT max(id) FROM Person").fetch_one(db).await.unwrap();
        match max {
            Some(n) => n + 1,
            _ => 1,
        }
    }

    async fn build(db: &Db, age: u16, country_id: CountryId, gender: Gender) -> Self {
        let (forename, surname) = Country::fetch_from_db(db, country_id).await.generate_name(gender);
        return Self {
            id: Self::next_id(db).await,
            birthday: database::get_today(db).await.checked_sub(Duration::days(age as i64)).unwrap(),
            country_id: country_id,
            gender: gender,
            forename: forename,
            surname: surname,
            is_active: true,

            ..Default::default()
        };
    }

    // Get the total weight and weight of each country.
    pub async fn country_weights(db: &Db) -> (u32, Vec<(CountryId, u32)>) {
        let countries = Country::fetch_all(db).await;

        let mut country_weights = Vec::new();
        let mut total_weight = 0;
        for country in countries {
            let weight = match country.name == "Finland" {
                // Making Finns more likely to appear in what tries to emulate some kind of a Finnish league.
                true => country.get_combined_name_weight() * 20,
                _ => country.get_combined_name_weight(),
            };

            total_weight += weight;
            country_weights.push((country.id, weight));
        }

        return (total_weight, country_weights);
    }

    // Make a random person.
    pub async fn create(db: &Db, country_weights: &[(CountryId, u32)], total_weight: u32, min_age: u8, max_age: u8, gender: Gender) -> Self {
        let min_days = years_to_days(min_age);
        let max_days = years_to_days(max_age);

        let age = rand::random_range(min_days..=max_days);

        let random = rand::random_range(0..total_weight);
        let mut counter = 0;
        let mut country_id = 0;
        for (id, weight) in country_weights {
            counter += weight;

            if random < counter {
                country_id = *id;
                break;
            }
        }

        let person = Self::build(db, age, country_id, gender).await;
        person.save(db).await;
        return person;
    }

    async fn save(&self, db: &Db) {
        sqlx::query(
            "INSERT INTO Person
            (id, forename, surname, gender, country_id, birthday, is_active)
            VALUES ($1, $2, $3, $4, $5, $6, $7)"
        ).bind(self.id)
        .bind(self.forename.as_str())
        .bind(self.surname.as_str())
        .bind(self.gender)
        .bind(self.country_id)
        .bind(self.birthday)
        .bind(self.is_active)
        .execute(db).await.unwrap();
    }

    // Change the person entry in the database to inactive and remove all contracts and offers.
    pub async fn retire(&self, db: &Db) {
        sqlx::raw_sql(format!(
            "UPDATE Person SET is_active = FALSE
            WHERE id = {};
            DELETE FROM Contracts WHERE person_id = {}", self.id, self.id
        ).as_str())
        .execute(db).await.unwrap();
    }

    async fn country(&self, db: &Db) -> Country {
        Country::fetch_from_db(db, self.country_id).await
    }

    pub fn full_name(&self) -> String {
        format!("{} {}", self.forename, self.surname)
    }

    fn _initial_and_surname(&self) -> String {
        format!("{}. {}", self.forename.chars().nth(0).unwrap(), self.surname)
    }

    // Get the age of the person as a duration.
    async fn age(&self, db: &Db) -> Duration {
        return database::get_today(db).await - self.birthday;
    }

    // Get the person's age in days.
    async fn age_in_days(&self, db: &Db) -> u16 {
        return self.age(db).await.whole_days() as u16;
    }

    // Get the person's age in years.
    async fn age_in_years(&self, db: &Db) -> i8 {
        return get_years_between(self.birthday, database::get_today(db).await);
    }

    // Get the contract of the person.
    pub async fn contract(&self, db: &Db) -> Option<Contract> {
        sqlx::query_as(
            "SELECT * FROM Contract
            WHERE person_id = $1 AND is_signed = TRUE"
        ).bind(self.id)
        .fetch_optional(db).await.unwrap()
    }

    async fn contract_offers(&self, db: &Db) -> Vec<Contract> {
        sqlx::query_as(
            "SELECT * FROM Contract
            WHERE person_id = $1 AND is_signed = FALSE"
        ).bind(self.id)
        .fetch_all(db).await.unwrap()
    }

    // Get the amount of contract offers the person has.
    pub async fn no_of_offers(&self, db: &Db) -> u8 {
        sqlx::query_scalar(
            "SELECT COUNT(*) FROM Contract
            WHERE person_id = $1 AND is_signed = FALSE"
        ).bind(self.id)
        .fetch_one(db).await.unwrap()
    }

    // Get a package of the person.
    async fn package(&self, db: &Db) -> serde_json::Value {
        let contract = match self.contract(db).await {
            Some(contract) => Some(contract.package(db).await),
            _ => None
        };

        let mut contract_offers = Vec::new();
        for offer in self.contract_offers(db).await {
            contract_offers.push(offer.package(db).await);
        }

        json!({
            "name": self.full_name(),
            "country": self.country(db).await.name_and_flag_package(),
            "age": self.age_in_years(db).await,
            "birthday": date_to_string(self.birthday),
            "contract": contract,
            "offers": contract_offers
        })
    }
}

// Functional.
impl Person {
    // Determine if the person is going to sign a contract now.
    // Very simple still.
    pub async fn decide_to_sign(&self, db: &Db) -> bool {
        let offers = self.contract_offers(db).await;
        if offers.is_empty() { return false; }
        let days_since_earliest_offer = offers[0].days_expired(db).await;

        // Random chance for the person to sign, grows more likely the more time passes.
        // Guaranteed to sign after 10 days.
        return rand::random_range(1..10) < days_since_earliest_offer;
    }
}

#[derive(Copy, Clone, Debug)]
#[derive(Serialize, Deserialize)]
#[derive(sqlx::Type)]
pub enum ContractRole {
    Player,
    Manager,
}

// Contract a person has with a club.
#[derive(Clone, Debug)]
#[derive(FromRow)]
pub struct Contract {
    person_id: PersonId,
    pub team_id: TeamId,
    #[sqlx(rename = "begin_date")]
    start_date: Date,
    end_date: Date,
    role: ContractRole,
    is_signed: bool,
}

impl Contract {
    // Create a contract.
    pub async fn build_and_save(db: &Db, person_id: PersonId, team_id: TeamId, start_date: Date, end_date: Date, role: ContractRole, is_signed: bool) -> Self {
        let contract = Self {
            person_id,
            team_id,
            start_date,
            end_date,
            role,
            is_signed,
        };

        contract.save_new(db).await;
        return contract;
    }

    // Create a contract based on the team and how many years it should last.
    pub async fn build_from_years(db: &Db, person_id: PersonId, team: &Team, years: i32, role: ContractRole, is_signed: bool) -> Self {
        let comp = Competition::fetch_from_db(db, team.primary_comp_id).await;
        let end_date = comp.season_window.end.get_previous_date_with_year_offset(db, years).await;

        return Self::build_and_save(db, person_id, team.id, database::get_today(db).await, end_date, role, is_signed).await;
    }

    async fn save_new(&self, db: &Db) {
        sqlx::query(
            "INSERT INTO Contract
            (person_id, team_id, begin_date, end_date, role, is_signed)
            VALUES ($1, $2, $3, $4, $5, $6)"
        ).bind(self.person_id)
        .bind(self.team_id)
        .bind(self.start_date)
        .bind(self.end_date)
        .bind(self.role)
        .bind(self.is_signed)
        .execute(db).await.unwrap();
    }

    // Delete all expired.
    pub async fn delete_expired(db: &Db) {
        sqlx::query(
            // Comparing each contract end date to the current day and deleting if the end date has passed.
            "DELETE FROM Contract WHERE unixepoch(end_date) <= (
                SELECT unixepoch(value_data) FROM KeyValue
                WHERE key_name = 'today'
            )"
        ).execute(db).await.unwrap();
    }

    // Change this contract from unsigned to signed and set the start date to today.
    async fn sign(&self, db: &Db) {
        sqlx::query(
            "UPDATE Contract
            SET is_signed = TRUE, begin_date = (
                SELECT value_data FROM KeyValue WHERE key_name = 'today'
            )
            WHERE person_id = $1 AND team_id = $2"
        ).bind(self.person_id)
        .bind(self.team_id)
        .execute(db).await.unwrap();
    }

    // Get the team of the contract.
    async fn team(&self, db: &Db) -> Team {
        sqlx::query_as(
            "SELECT * FROM Team WHERE id = $1"
        ).bind(self.team_id)
        .fetch_one(db).await.unwrap()
    }

    // How many days there are left of the contract.
    async fn _days_left(&self, db: &Db) -> i64 {
        self._duration_left(db).await.whole_days()
    }

    // How many days have expired from the contract.
    async fn days_expired(&self, db: &Db) -> i64 {
        return self.duration_expired(db).await.whole_days()
    }

    // How many seasons there are left of the contract.
    // Note that 1 means less than a year left of the contract!
    async fn seasons_left(&self, db: &Db) -> i8 {
        return get_years_between(database::get_today(db).await, self.end_date) + 1;
    }

    // Get how much is left of the contract.
    async fn _duration_left(&self, db: &Db) -> Duration {
        return self.end_date - database::get_today(db).await;
    }

    // Get how much has expired of the contract.
    async fn duration_expired(&self, db: &Db) -> Duration {
        return database::get_today(db).await - self.start_date;
    }

    // Get relevant information for frontend.
    async fn package(&self, db: &Db) -> serde_json::Value {
        json!({
            "start_date": date_to_string(self.start_date),
            "end_date": date_to_string(self.end_date),
            "seasons_left": self.seasons_left(db).await,
            "team": self.team(db).await.contract_package()
        })
    }
}