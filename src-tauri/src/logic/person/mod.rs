pub mod player;

pub mod attribute;
pub mod contract;
pub mod manager;

use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::FromRow;
use time::{Date, Duration};

use crate::logic::{app_data::AppData, country::Country, person::contract::Contract, time::{date_to_string, years_between, years_to_days}, types::{CountryId, Db, PersonId}};

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
    pub gender: Gender,
    pub country_id: CountryId,
    pub birthday: Date,
    pub is_active: bool,

    // Generated
    full_name: String,
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
            full_name: String::default(),
        }
    }
}

impl Person {
    async fn build(db: &Db, today: Date, age: u16, country_id: CountryId, gender: Gender) -> Self {
        let (forename, surname) = Country::fetch_from_db(db, country_id).await.generate_name(gender);
        return Self {
            birthday: today.checked_sub(Duration::days(age as i64)).unwrap(),
            country_id: country_id,
            gender: gender,
            forename: forename,
            surname: surname,
            is_active: true,

            ..Default::default()
        };
    }

    // Make a random person.
    pub async fn create(data: &AppData, today: Date, min_age: u8, max_age: u8, gender: Gender) -> Self {
        let min_days = years_to_days(min_age);
        let max_days = years_to_days(max_age);

        let age = rand::random_range(min_days..=max_days);

        let random = rand::random_range(0..data.country_weights.total);
        let mut counter = 0;
        let mut country_id = 0;
        for (id, weight) in data.country_weights.weights.iter() {
            counter += weight;

            if random < counter {
                country_id = *id;
                break;
            }
        }

        let mut person = Self::build(&data.db, today, age, country_id, gender).await;
        person.save(&data.db).await;
        return person;
    }

    async fn country(&self, db: &Db) -> Country {
        Country::fetch_from_db(db, self.country_id).await
    }

    fn _initial_and_surname(&self) -> String {
        format!("{}. {}", self.forename.chars().nth(0).unwrap(), self.surname)
    }

    // Get the age of the person as a duration.
    fn age(&self, today: Date) -> Duration {
        return today - self.birthday;
    }

    // Get the person's age in days.
    fn age_in_days(&self, today: Date) -> u16 {
        return self.age(today).whole_days() as u16;
    }

    // Get the person's age in years.
    fn age_in_years(&self, today: Date) -> i8 {
        return years_between(self.birthday, today);
    }

    // Get a package of the person.
    async fn package(&self, db: &Db, today: Date) -> serde_json::Value {
        let contract = match self.contract(db).await {
            Some(contract) => Some(contract.package(db, today).await),
            _ => None
        };

        let mut contract_offers = Vec::new();
        for offer in self.contract_offers(db).await {
            contract_offers.push(offer.package(db, today).await);
        }

        json!({
            "name": self.full_name,
            "country": self.country(db).await.name_and_flag_package(),
            "age": self.age_in_years(today),
            "birthday": date_to_string(self.birthday),
            "contract": contract,
            "offers": contract_offers
        })
    }

    // Determine if the person is going to sign a contract now.
    // Very simple still.
    pub fn decide_to_sign(&self, today: Date, offers: &[Contract]) -> bool {
        if offers.is_empty() { return false; }
        let days_since_earliest_offer = offers[0].days_expired(today);

        // Random chance for the person to sign, grows more likely the more time passes.
        // Guaranteed to sign after 10 days.
        return rand::random_range(1..10) < days_since_earliest_offer;
    }
}