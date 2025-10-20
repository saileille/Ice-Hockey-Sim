pub mod player;
pub mod manager;

use rand;
use serde_json::json;
use time::{Date, Duration};

use crate::{
    competition::Competition, country::Country, database::{COUNTRIES, TODAY}, team::Team, time::{date_to_db_string, db_string_to_date}, types::{convert, CountryId, TeamId}
};

#[derive(Debug)]
#[derive(Default, Clone, PartialEq)]
enum Gender {
    #[default] Null,
    Male,
    Female,
}

#[derive(Debug)]
#[derive(Default, Clone)]
pub struct Person {
    pub forename: String,
    pub surname: String,
    gender: Gender,
    country_id: CountryId,
    pub contract: Option<Contract>,
    pub contract_offers: Vec<Contract>,
}

// Basics.
impl Person {
    fn build(country_id: CountryId, gender: Gender) -> Self {
        let mut person = Person::default();
        person.country_id = country_id;
        (person.forename, person.surname) = Country::fetch_from_db(&person.country_id).generate_name();

        person.gender = gender;

        return person;
    }

    // Make a random person (male).
    pub fn build_random() -> Self {
        // First determining the person's nationality with weighted random.
        let countries = COUNTRIES.lock().unwrap().clone();
        let mut country_weights = Vec::new();
        let mut total_weight = 0;
        for country in countries.values() {
            let weight = match country.name == "Finland" {
                // Making Finns more likely to appear in what tries to emulate some kind of a Finnish league.
                true => country.forenames.total_weight * 20,
                _ => country.forenames.total_weight,
            };

            total_weight += weight;
            country_weights.push((country.id, weight));
        }

        let random = rand::random_range(0..total_weight);
        let mut counter = 0;
        let mut country_id = 0;
        for (id, weight) in country_weights {
            counter += weight;

            if random < counter {
                country_id = id;
                break;
            }
        }

        return Self::build(country_id, Gender::Male);
    }

    // Check if the person in question does not have default traits.
    fn is_valid(&self) -> bool {
        self.forename != String::default() &&
        self.surname != String::default() &&
        self.gender != Gender::default() &&
        self.country_id != 0
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
}

// Functional.
impl Person {
    // Delete the person's contract if it has ended.
    // Return whether the contract ended or not.
    pub fn check_if_contract_expired(&mut self) -> bool {
        if self.contract.is_none() { return false; }
        return self.contract.as_ref().unwrap().check_if_expired();
    }

    // Determine if the person is going to sign a contract now.
    // Very simple still.
    pub fn decide_to_sign(&self) -> bool {
        if self.contract_offers.is_empty() { return false; }
        let days_since_earliest_offer = self.contract_offers[0].get_days_expired();

        // Random chance for the person to sign, grows more likely the more time passes.
        // Guaranteed to sign after 10 days.
        return rand::random_range(1..10) < days_since_earliest_offer;
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
        let end_date = comp.season_window.end.get_previous_date_with_year_offset(years);

        return Self::build(&date_to_db_string(today), &date_to_db_string(&end_date), team.id);
    }

    // Get the team of the contract.
    fn get_team(&self) -> Team {
        Team::fetch_from_db(&self.team_id)
    }

    // How many days there are left of the contract.
    fn get_days_left(&self) -> i64 {
        self.get_duration_left().whole_days()
    }

    // How many days have expired from the contract.
    fn get_days_expired(&self) -> i64 {
        return self.get_duration_expired().whole_days()
    }

    // How many seasons there are left of the contract.
    // Note that 1 means less than a year left of the contract!
    fn get_seasons_left(&self) -> i8 {
        let today = TODAY.lock().unwrap().clone();
        let end_date = db_string_to_date(&self.end_date);
        let years = convert::i32_to_i8(1 + end_date.year() - today.year());

        match end_date.month() as i8 - today.month() as i8 {
            1..=i8::MAX => years,
            0 => match end_date.day() as i8 - today.day() as i8 {
                1..=i8::MAX => years,
                _ => years - 1
            },
            _ => years - 1
        }
    }

    // Get how much is left of the contract.
    fn get_duration_left(&self) -> Duration {
        let today = TODAY.lock().unwrap().clone();
        return db_string_to_date(&self.end_date) - today;
    }

    // Get how much has expired of the contract.
    fn get_duration_expired(&self) -> Duration {
        let today = TODAY.lock().unwrap().clone();
        return today - db_string_to_date(&self.start_date);
    }

    // Check if the contract has expired.
    pub fn check_if_expired(&self) -> bool {
        return self.get_days_left() <= 0
    }

    // Get relevant information for a person screen.
    fn get_person_screen_json(&self) -> serde_json::Value {
        json!({
            "start_date": self.start_date,
            "end_date": self.end_date,
            "seasons_left": self.get_seasons_left(),
            "team": self.get_team().get_player_screen_json()
        })
    }
}