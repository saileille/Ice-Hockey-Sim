pub mod player;
pub mod manager;
pub mod attribute;

use std::collections::HashMap;

use rand::{self, Rng, rngs::ThreadRng};
use serde_json::json;
use time::{Date, Duration};

use crate::{
    competition::Competition, country::Country, database::COUNTRIES, team::Team, time::{date_to_db_string, db_string_to_date, get_years_between, years_to_days}, types::{CountryId, TeamId}
};

#[derive(Eq, Hash)]
#[derive(Debug)]
#[derive(Default, Clone, PartialEq)]
pub enum Gender {
    #[default] Null,
    Male,
    Female,
}

#[derive(Debug)]
#[derive(Clone)]
pub struct Person {
    pub forename: String,
    pub surname: String,
    gender: Gender,
    country_id: CountryId,
    pub contract: Option<Contract>,
    pub contract_offers: Vec<Contract>,
    birthday: Date,
    pub is_active: bool,
}

impl Default for Person {
    fn default() -> Self {
        Self {
            forename: String::default(),
            surname: String::default(),
            gender: Gender::default(),
            country_id: CountryId::default(),
            contract: None,
            contract_offers: Vec::default(),
            birthday: Date::MIN,
            is_active: bool::default(),
        }
    }
}

// Basics.
impl Person {
    fn build(today: &Date, rng: &mut ThreadRng, age: u16, country_id: CountryId, gender: Gender) -> Self {
        let (forename, surname) = Country::fetch_from_db(&country_id).generate_name(&gender, rng);
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
    pub fn create(countries: &HashMap<CountryId, Country>, today: &Date, rng: &mut ThreadRng, min_age: u8, max_age: u8, gender: Gender) -> Self {
        let min_days = years_to_days(min_age);
        let max_days = years_to_days(max_age);

        let age = rng.random_range(min_days..=max_days);

        // First determining the person's nationality with weighted random.
        let mut country_weights = Vec::new();
        let mut total_weight = 0;
        for country in countries.values() {
            let weight = match country.name == "Finland" {
                // Making Finns more likely to appear in what tries to emulate some kind of a Finnish league.
                true => country.get_combined_name_weight() * 20,
                _ => country.get_combined_name_weight(),
            };

            total_weight += weight;
            country_weights.push((country.id, weight));
        }

        let random = rng.random_range(0..total_weight);
        let mut counter = 0;
        let mut country_id = 0;
        for (id, weight) in country_weights {
            counter += weight;

            if random < counter {
                country_id = id;
                break;
            }
        }

        return Self::build(today, rng, age, country_id, gender);
    }

    // Get the person's country as an object.
    fn get_country(&self) -> Country {
        Country::fetch_from_db(&self.country_id)
    }

    pub fn get_full_name(&self) -> String {
        format!("{} {}", self.forename, self.surname)
    }

    fn get_initial_and_surname(&self) -> String {
        format!("{}. {}", self.forename.chars().nth(0).unwrap(), self.surname)
    }

    // Get the age of the person as a duration.
    fn get_age(&self, today: &Date) -> Duration {
        return *today - self.birthday;
    }

    // Get the person's age in days.
    fn get_age_days(&self, today: &Date) -> u16 {
        return self.get_age(today).whole_days() as u16;
    }

    // Get the person's age in years.
    fn get_age_years(&self, today: &Date) -> i8 {
        return get_years_between(&self.birthday, &today);
    }

    // Get a package of the person.
    fn get_package(&self, today: &Date) -> serde_json::Value {
        let contract = match self.contract.as_ref() {
            Some(contract) => Some(contract.get_package(today)),
            _ => None
        };

        let contract_offers: Vec<serde_json::Value> = self.contract_offers.iter().map(|a| a.get_package(today)).collect();

        json!({
            "name": self.get_full_name(),
            "country": self.get_country().get_name_and_flag_package(),
            "age": self.get_age_years(today),
            "birthday": date_to_db_string(&self.birthday),
            "contract": contract,
            "offers": contract_offers
        })
    }
}

// Functional.
impl Person {
    // Delete the person's contract if it has ended.
    // Return whether the contract ended or not.
    pub fn check_if_contract_expired(&mut self, today: &Date) -> bool {
        if self.contract.is_none() { return false; }
        return self.contract.as_ref().unwrap().check_if_expired(today);
    }

    // Determine if the person is going to sign a contract now.
    // Very simple still.
    pub fn decide_to_sign(&self, today: &Date, rng: &mut ThreadRng) -> bool {
        if self.contract_offers.is_empty() { return false; }
        let days_since_earliest_offer = self.contract_offers[0].get_days_expired(today);

        // Random chance for the person to sign, grows more likely the more time passes.
        // Guaranteed to sign after 10 days.
        return rng.random_range(1..10) < days_since_earliest_offer;
    }
}

// Contract a person has with a club.
#[derive(Debug)]
#[derive(Clone)]
pub struct Contract {
    start_date: String,
    end_date: String,
    pub team_id: TeamId
}

impl Contract {
    // Create a contract.
    pub fn build(start_date: &str, end_date: &str, team_id: TeamId) -> Self {
        Self {
            start_date: start_date.to_string(),
            end_date: end_date.to_string(),
            team_id: team_id,
        }
    }

    // Create a contract based on the team and how many years it should last.
    pub fn build_from_years(team: &Team, today: &Date, years: i32) -> Self {
        let comp = Competition::fetch_from_db(&team.primary_comp_id);
        let end_date = comp.season_window.end.get_previous_date_with_year_offset(years, today);

        return Self::build(&date_to_db_string(today), &date_to_db_string(&end_date), team.id);
    }

    // Get the team of the contract.
    fn get_team(&self) -> Team {
        Team::fetch_from_db(&self.team_id)
    }

    // How many days there are left of the contract.
    fn get_days_left(&self, today: &Date) -> i64 {
        self.get_duration_left(today).whole_days()
    }

    // How many days have expired from the contract.
    fn get_days_expired(&self, today: &Date) -> i64 {
        return self.get_duration_expired(today).whole_days()
    }

    // How many seasons there are left of the contract.
    // Note that 1 means less than a year left of the contract!
    fn get_seasons_left(&self, today: &Date) -> i8 {
        let end_date = db_string_to_date(&self.end_date);
        return get_years_between(&today, &end_date) + 1;
    }

    // Get how much is left of the contract.
    fn get_duration_left(&self, today: &Date) -> Duration {
        return db_string_to_date(&self.end_date) - *today;
    }

    // Get how much has expired of the contract.
    fn get_duration_expired(&self, today: &Date) -> Duration {
        return *today - db_string_to_date(&self.start_date);
    }

    // Check if the contract has expired.
    pub fn check_if_expired(&self, today: &Date) -> bool {
        return self.get_days_left(today) <= 0
    }

    // Get relevant information for frontend.
    fn get_package(&self, today: &Date) -> serde_json::Value {
        json!({
            "start_date": self.start_date,
            "end_date": self.end_date,
            "seasons_left": self.get_seasons_left(today),
            "team": self.get_team().get_contract_package()
        })
    }
}