pub mod player;

use crate::{
    types::CountryId,
    country::Country
};

#[derive(Default, Clone, PartialEq)]
enum Gender {
    #[default] Null,
    Male,
    Female,
}

#[derive(Default, Clone)]
pub struct Person {
    forename: String,
    surname: String,
    gender: Gender,
    country_id: CountryId,
}

// Basics.
impl Person {
    fn build(country_id: CountryId, gender: Gender) -> Self {
        let mut person: Person = Person::default();
        person.country_id = country_id;
        (person.forename, person.surname) = Country::fetch_from_db(&person.country_id).generate_name();

        person.gender = gender;

        return person;
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
}

// Functional.
impl Person {
    pub fn get_full_name(&self) -> String {
        format!("{} {}", self.forename, self.surname)
    }

    fn get_initial_and_surname(&self) -> String {
        format!("{}. {}", self.forename.chars().nth(0).unwrap(), self.surname)
    }
}