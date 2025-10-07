// Countries and such.
use std::collections::HashMap;
use std::ops::Range;
use rand::random_range;

use crate::types::CountryId;
use crate::database::COUNTRIES;
use crate::io::load_country_names;

#[derive(Default, Clone)]
pub struct Country {
    pub id: CountryId,
    name: String,
    forenames: NamePool,
    surnames: NamePool,
}

impl Country {  // Basics.
    // Validate an ID.
    fn create_id(&mut self, id: usize) {
        self.id = match id.try_into() {
            Ok(id) => id,
            Err(e) => panic!("{e}"),
        };
    }

    // Build a country element.
    fn build<S: AsRef<str>>(name: S) -> Self {
        let mut country: Self = Self::default();
        country.name = String::from(name.as_ref());
        country.assign_names();
        return country;
    }

    // Build a Country element and store it in the database. Return the created element.
    pub fn build_and_save<S: AsRef<str>>(name: S) -> Self {
        let mut country: Self = Self::build(name.as_ref());
        country.create_id(COUNTRIES.lock().unwrap().len() + 1);

        country.update_to_db();
        return country;
    }

    // Get a Country from the database.
    pub fn fetch_from_db(id: &CountryId) -> Self {
        COUNTRIES.lock().unwrap().get(id).expect(&format!("no Country with id {id}")).clone()
    }

    // Update the Country to database.
    pub fn update_to_db(&self) {
        COUNTRIES.lock()
            .expect(&format!("something went wrong when trying to update Country {}: {} to COUNTRIES", self.id, self.name))
            .insert(self.id, self.clone());
    }

    // Get a Country from the database with the given name.
    pub fn fetch_from_db_with_name<S: AsRef<str>>(name: S) -> Self {
        let name_ref: &str = name.as_ref();

        for country in COUNTRIES.lock().unwrap().values() {
            if country.name == name_ref {
                return country.clone();
            }
        }

        panic!("country with name {name_ref} does not exist!");
    }

    // Make sure the country does not contain illegal values.
    fn is_valid(&self) -> bool {
        self.id != 0 &&
        self.name != String::default() &&
        self.forenames.is_valid() &&
        self.surnames.is_valid()
    }
}

impl Country {
    // Assign surnames and forenames to the country.
    fn assign_names(&mut self) {
        let names: HashMap<String, HashMap<String, u16>> = load_country_names(&self.name);
        self.forenames.populate(names.get("forenames").unwrap().clone());
        self.surnames.populate(names.get("surnames").unwrap().clone());
    }

    // Generate a name from the country's name databases.
    pub fn generate_name(&self) -> (String, String) {
        (self.forenames.draw_name(), self.surnames.draw_name())
    }
}

// Namepool with names and weights.
#[derive(Default, Clone, PartialEq)]
struct NamePool {
    names: Vec<String>,
    weights: Vec<u16>,
    total_weight: usize,
}

impl NamePool { // Basics.
    // Check that the NamePool does not contain illegal values.
    fn is_valid(&self) -> bool {
        if self.names.len() == 0 || self.names.len() != self.weights.len() {
            return false;
        }

        let mut sum: usize = 0;
        for weight in self.weights.iter() {
            sum += *weight as usize;
        }

        return self.total_weight == sum;
    }

    // Populate the namepool with names and calculate total_weight.
    fn populate(&mut self, name_map: HashMap<String, u16>) {
        self.names = Vec::new();
        self.weights = Vec::new();
        self.total_weight = 0;

        for (name, weight) in name_map {
            self.names.push(name);
            self.weights.push(weight);
            self.total_weight += weight as usize;
        }
    }
}

impl NamePool {
    // Draw a single name from the name pool.
    fn draw_name(&self) -> String {
        return self.names[self.draw_index()].clone()
    }

    // Get a random index of the weights/names vector.
    fn draw_index(&self) -> usize {
        let range: Range<usize> = Range { start: 0, end: self.total_weight };
        let random: usize = random_range(range);

        let mut counter: usize = 0;
        for (i, weight) in self.weights.iter().enumerate() {
            counter += *weight as usize;
            if random < counter {
                return i;
            }
        }

        return self.weights.len();  // This should *never* happen.
    }
}
