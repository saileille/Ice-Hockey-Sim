// Input/output logic.
use std::{collections::HashMap, fs::{self, ReadDir}, io::{self, Read}, path::PathBuf};


use crate::database::{COUNTRY_FLAG_DIR, PEOPLE_NAME_DIR};

/*static PATHS: [&str; 2] = [
    "./json/names", // Windows
    "..usr/lib/ice-hockey-sim/json/names",  // Linux
];*/

// Get a file, or error if it does not exist.
fn get_file(path: &str) -> io::Result<fs::File> {
    return fs::File::open(path);
}

// Read a JSON file and return it as a string.
fn read_json_file(path: &PathBuf) -> io::Result<String> {
    let mut json = String::new();
    // let file_result = File::open(path.as_ref());
    let mut file = match fs::File::open(path) {
        Ok(f) => f,
        Err(e) => return Err(e),
    };

    file.read_to_string(&mut json).unwrap();
    return Ok(json);
}

// Load names of a specific country.
pub fn load_country_names(country: &str) -> HashMap<String, HashMap<String, HashMap<String, u16>>> {
    let path_buf = PathBuf::from(PEOPLE_NAME_DIR.lock().unwrap().clone()).join(format!("{country}.json"));

    let json = read_json_file(&path_buf).unwrap();
    return serde_json::from_str(&json).unwrap();
}

// Get the flag path for a country.
pub fn get_flag_path(country: &str) -> Option<String> {
    let path_buf = PathBuf::from(COUNTRY_FLAG_DIR.lock().unwrap().clone()).join(format!("{country}.svg"));
    match path_buf.exists() {
        true => Some(path_buf.to_str().unwrap().to_string()),
        _ => None,
    }
}

// Get a readable directory from a string.
pub fn get_read_dir(path: &PathBuf) -> ReadDir {
    return fs::read_dir(path).expect(&format!("no path {}", path.to_str().unwrap()));
}

// Function for listing all JSON files in the names folder.
// Used for generating countries in the database.
pub fn get_countries_from_name_files() -> Vec<String> {
    let path_buf = PathBuf::from(PEOPLE_NAME_DIR.lock().unwrap().clone());

    let dir = get_read_dir(&path_buf);
    let mut countries = Vec::new();

    for entry in dir {
        let filename = format!("{}", entry.unwrap().file_name().display());
        if !filename.ends_with(".json") { continue }

        let country_name = String::from(&filename[0..filename.len() - 5]);
        countries.push(country_name);
    }

    return countries;
}