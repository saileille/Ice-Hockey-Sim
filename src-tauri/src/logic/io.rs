// Input/output logic.
use std::{collections::HashMap, fs::{self, ReadDir, remove_file}, io::{self, Read}, path::PathBuf};

use crate::logic::app_data::Directories;

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

// Remove the db file.
pub fn remove_db(directories: &Directories) {
    let path_buf = PathBuf::from(directories.db.as_str()).join("db.db");
    let path = path_buf.to_str().unwrap();
    match remove_file(path) {
        Ok(_) => println!("Removing the database was successful."),
        Err(e) => println!("Removing the database was unsuccessful. Error: {e}"),
    }
}

// Load names of a specific country.
pub fn load_country_names(directories: &Directories, country: &str) -> HashMap<String, HashMap<String, HashMap<String, u16>>> {
    let path_buf = PathBuf::from(directories.names.as_str()).join(format!("{country}.json"));

    let json = read_json_file(&path_buf).unwrap();
    return serde_json::from_str(&json).unwrap();
}

// Get the flag path for a country.
pub fn get_flag_path(directories: &Directories, country: &str) -> Option<String> {
    let path_buf = PathBuf::from(directories.flags.as_str()).join(format!("{country}.svg"));
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
pub fn get_countries_from_name_files(directories: &Directories) -> Vec<String> {
    let path_buf = PathBuf::from(directories.names.as_str());

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