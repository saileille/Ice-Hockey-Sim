use serde::Serialize;
use sqlx::FromRow;

#[derive(Serialize, FromRow)]
pub struct CountryPackage {
    #[sqlx(rename = "country_name")]
    name: String,
    flag_path: String,
}