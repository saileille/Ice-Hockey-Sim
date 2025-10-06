// Input/output logic.
use std::collections::HashMap;
use std::fs::{File, ReadDir, read_dir};
use std::io::Read;

fn read_json_file<S: AsRef<str>>(path: S) -> String {
    // Read a JSON file and return it as a string.
    let mut json: String = String::new();
    let mut file: File = File::open(path.as_ref()).unwrap();

    file.read_to_string(&mut json).unwrap();
    return json
}

pub fn load_country_names<S: AsRef<str>>(country: S) -> HashMap<String, HashMap<String, u16>> {
    // Load names of a specific country.
    let json: String = read_json_file(format!("./json/names/{}.json", country.as_ref()));
    let names: HashMap<String, HashMap<String, u16>> = serde_json::from_str(&json).unwrap();
    return names
}

// Function for listing all JSON files in the names folder.
// Used for generating countries in the database.
pub fn get_countries_from_name_files() -> Vec<String> {
    let paths: ReadDir = read_dir("./json/names/").unwrap();
    let mut countries: Vec<String> = Vec::new();

    for path in paths {
        let filename: String = format!("{}", path.unwrap().file_name().display());
        if !filename.ends_with(".json") { continue }

        let country_name: String = String::from(&filename[0..filename.len() - 5]);
        countries.push(country_name);
    }

    return countries;
}