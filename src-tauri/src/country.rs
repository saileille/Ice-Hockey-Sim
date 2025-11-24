// Countries and such.
use std::collections::HashMap;
use rand::{Rng, rngs::ThreadRng};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::FromRow;

use crate::{app_data::Directories, io::{get_flag_path, load_country_names}, person::Gender, types::{CountryId, CountryNamePool, Db}};

#[derive(Default, Clone)]
#[derive(FromRow)]
pub struct Country {
    pub id: CountryId,
    #[sqlx(rename = "country_name")]
    pub name: String,
    #[sqlx(json)]
    names: CountryNamePool,
    flag_path: Option<String>,
}

// Basics.
impl Country {
    // Get the next ID to use.
    async fn next_id(db: &Db) -> CountryId {
        let max: Option<CountryId> = sqlx::query_scalar("SELECT max(id) FROM Country").fetch_one(db).await.unwrap();
        match max {
            Some(n) => n + 1,
            _ => 1,
        }
    }

    // Build a country element.
    async fn build(directories: &Directories, db: &Db, name: &str) -> Self {
        let mut country = Self {
            id: Self::next_id(db).await,
            name: name.to_string(),
            flag_path: get_flag_path(directories, name),

            ..Default::default()
        };

        country.assign_names(directories);
        return country;
    }

    // Build a Country element and store it in the database. Return the created element.
    pub async fn build_and_save(directories: &Directories, db: &Db, name: &str) -> Self {
        let country = Self::build(directories, db, name).await;
        country.save(db).await;
        return country;
    }

    // Get a Country from the database.
    pub async fn fetch_from_db(db: &Db, id: CountryId) -> Self {
        sqlx::query_as(
            "SELECT * FROM Country WHERE id = $1"
        ).bind(id)
        .fetch_one(db).await.unwrap()
    }

    // Fetch ALL from the database.
    pub async fn fetch_all(db: &Db) -> Vec<Self> {
        sqlx::query_as(
            "SELECT * FROM Country"
        )
        .fetch_all(db).await.unwrap()
    }

    // Save the Country to database.
    pub async fn save(&self, db: &Db) {
        sqlx::query(
            "INSERT INTO Country
            (id, country_name, names, flag_path)
            VALUES ($1, $2, $3, $4)"
        ).bind(self.id)
        .bind(&self.name)
        .bind(serde_json::to_string(&self.names).unwrap())
        .bind(&self.flag_path)
        .execute(db).await.unwrap();
    }

    pub fn name_and_flag_package(&self) -> serde_json::Value {
        json!({
            "name": self.name,
            "flag_path": self.flag_path,
        })
    }
}

impl Country {
    // Assign surnames and forenames to the country.
    fn assign_names(&mut self, directories: &Directories) {
        let json = load_country_names(directories, &self.name);
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
    pub fn generate_name(&self, gender: Gender) -> (String, String) {
        let mut rng = rand::rng();
        let forename = self.names.get(&gender).unwrap().get("forenames").unwrap().draw_name(&mut rng);
        let surname = self.names.get(&gender).unwrap().get("surnames").unwrap().draw_name(&mut rng);

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
#[derive(Serialize, Deserialize)]
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