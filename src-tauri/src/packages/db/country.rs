use crate::logic::types::{CountryId, CountryNamePool};

struct Country {
    id: CountryId,
    country_name: String,
    names: CountryNamePool,
    flag_path: String,
}