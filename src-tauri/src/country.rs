// Countries and such.
use std::
    collections::HashMap
;
use rand::random_range;

use crate::{
    database::COUNTRIES, io::load_country_names, person::Gender, types::{CountryId, CountryNamePool}
};

#[derive(Default, Clone)]
pub struct Country {
    pub id: CountryId,
    pub name: String,
    names: CountryNamePool,
}

impl Country {  // Basics.
    // Validate an ID.
    fn create_id(&mut self, id: usize) {
        self.id = match id.try_into() {
            Ok(n) => n,
            Err(e) => panic!("{e}"),
        };
    }

    // Build a country element.
    fn build(name: &str) -> Self {
        let mut country = Self::default();
        country.name = name.to_string();
        country.assign_names();
        return country;
    }

    // Build a Country element and store it in the database. Return the created element.
    pub fn build_and_save(name: &str) -> Self {
        let mut country = Self::build(name);
        country.create_id(COUNTRIES.lock().unwrap().len() + 1);

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

    // Make sure the country does not contain illegal values.
    fn is_valid(&self) -> bool {
        self.id != 0 &&
        self.name != String::default() /*&&
        self.forenames.is_valid() &&
        self.surnames.is_valid()*/
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
    pub fn generate_name(&self, gender: &Gender) -> (String, String) {
        let forename = self.names.get(gender).unwrap().get("forenames").unwrap().draw_name();
        let surname = self.names.get(gender).unwrap().get("surnames").unwrap().draw_name();

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

    // Check that the NamePool does not contain illegal values.
    fn is_valid(&self) -> bool {
        if self.names.len() == 0 || self.names.len() != self.weights.len() {
            return false;
        }

        let mut sum = 0;
        for weight in self.weights.iter() {
            sum += *weight as u32;
        }

        return self.total_weight == sum;
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
    fn draw_name(&self) -> String {
        return self.names[self.draw_index()].clone()
    }

    // Get a random index of the weights/names vector.
    fn draw_index(&self) -> usize {
        let random = random_range(0..self.total_weight);
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
