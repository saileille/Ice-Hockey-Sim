// Countries and such.
use std::collections::HashMap;
use rand::{Rng, rngs::ThreadRng};

use crate::{database::COUNTRIES, io::load_country_names, person::Gender, types::{CountryId, CountryNamePool, convert}};

#[derive(Default, Clone)]
pub struct Country {
    pub id: CountryId,
    pub name: String,
    names: CountryNamePool,
}

// Basics.
impl Country {
    // Build a country element.
    fn build(name: &str) -> Self {
        let mut country = Self {
            id: convert::int::<usize, CountryId>(COUNTRIES.lock().unwrap().len() + 1),
            name: name.to_string(),
            ..Default::default()
        };

        country.assign_names();
        return country;
    }

    // Build a Country element and store it in the database. Return the created element.
    pub fn build_and_save(name: &str) -> Self {
        let country = Self::build(name);
        country.save();
        return country;
    }

    // Get a Country from the database.
    pub fn fetch_from_db(id: &CountryId) -> Self {
        COUNTRIES.lock().unwrap().get(id).expect(&format!("no Country with id {id}")).clone()
    }

    // Update the Country to database.
    pub fn save(&self) {
        COUNTRIES.lock().unwrap().insert(self.id, self.clone());
    }

    // Get a Country from the database with the given name.
    pub fn fetch_from_db_with_name(name: &str) -> Self {
        for country in COUNTRIES.lock().unwrap().values() {
            if country.name == name {
                return country.clone();
            }
        }

        panic!("country with name {name} does not exist!");
    }
}

impl Country {
    // Assign surnames and forenames to the country.
    fn assign_names(&mut self) {
        let json = load_country_names(&self.name);
        for (gender, gender_data) in json.iter() {
            let gender_enum;
            match gender.as_ref() {
                "male" => {
                    gender_enum = Gender::Male;
                    self.names.insert(gender_enum.clone(), HashMap::new());
                },
                "female" => {
                    gender_enum = Gender::Female;
                    self.names.insert(gender_enum.clone(), HashMap::new());
                },
                _ => panic!("no")
            };

            for (name_type, namedata) in gender_data.iter() {
                self.names.get_mut(&gender_enum).unwrap().insert(name_type.clone(), NamePool::build(namedata.clone()));
            }
        }
    }

    // Generate a name from the country's name databases.
    pub fn generate_name(&self, gender: &Gender, rng: &mut ThreadRng) -> (String, String) {
        let forename = self.names.get(gender).unwrap().get("forenames").unwrap().draw_name(rng);
        let surname = self.names.get(gender).unwrap().get("surnames").unwrap().draw_name(rng);

        (forename, surname)
    }

    // Get the combined name weight of the country's namepools.
    pub fn get_combined_name_weight(&self) -> u32 {
        self.names.get(&Gender::Male).unwrap().get("forenames").unwrap().total_weight +
        self.names.get(&Gender::Male).unwrap().get("surnames").unwrap().total_weight +
        self.names.get(&Gender::Female).unwrap().get("forenames").unwrap().total_weight +
        self.names.get(&Gender::Female).unwrap().get("surnames").unwrap().total_weight
    }
}

// Namepool with names and weights.
#[derive(Default, Clone)]
pub struct NamePool {
    names: Vec<String>,
    weights: Vec<u16>,
    pub total_weight: u32,
}
// Basics.
impl NamePool {
    pub fn build(names: HashMap<String, u16>) -> Self {
        let mut pool = Self::default();

        for (name, weight) in names.into_iter() {
            pool.names.push(name);
            pool.weights.push(weight);
        }

        pool.calculate_weight();
        return pool;
    }

    // Calculate the weight.
    fn calculate_weight(&mut self) {
        self.total_weight = 0;
        for weight in self.weights.iter() {
            self.total_weight += *weight as u32;
        }
    }
}

impl NamePool {
    // Draw a single name from the name pool.
    fn draw_name(&self, rng: &mut ThreadRng) -> String {
        return self.names[self.draw_index(rng)].clone()
    }

    // Get a random index of the weights/names vector.
    fn draw_index(&self, rng: &mut ThreadRng) -> usize {
        let random = rng.random_range(0..self.total_weight);
        let mut counter = 0;
        for (i, weight) in self.weights.iter().enumerate() {
            counter += *weight as u32;
            if random < counter {
                return i;
            }
        }

        return self.weights.len();  // This should *never* happen.
    }
}
