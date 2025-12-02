use crate::logic::{country::Country, types::{CountryId, Db}};

pub struct Directories {
    pub names: String,
    pub flags: String,
    pub db: String,
}

// Used for determining the frequency of each nationality.
#[derive(Default)]
pub struct CountryWeights {
    pub weights: Vec<(CountryId, u32)>,
    pub total: u32,
}

impl CountryWeights {
    pub async fn build(db: &Db) -> Self {
        let now = std::time::Instant::now();
        let countries = Country::fetch_all(db).await;

        let mut weights = Vec::new();
        let mut total = 0;
        for country in countries {
            let weight = match country.name == "Finland" {
                // Making Finns more likely to appear in what tries to emulate some kind of a Finnish league.
                true => country.get_combined_name_weight() * 20,
                false => country.get_combined_name_weight(),
            };

            total += weight;
            weights.push((country.id, weight));
        }

        println!("Created countryweights in {:.2?}", now.elapsed());
        return Self { weights, total };
    }
}

// For keeping track of stuff.
pub struct AppData {
    pub db: Db,
    pub directories: Directories,
    pub country_weights: CountryWeights,
}

impl AppData {
    // Build the thing.
    pub fn build(db: Db, directories: Directories) -> Self {
        Self {
            db,
            directories,
            country_weights: CountryWeights::default(),
        }
    }
}