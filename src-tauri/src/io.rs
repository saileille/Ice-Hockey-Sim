// Input/output logic.
use std::{collections::HashMap, fs::{self, ReadDir}, io::{self, Read}};

static PATHS: [&str; 2] = [
    "./json/names", // Windows
    "..usr/lib/ice-hockey-sim/json/names",  // Linux
];

// Get a file, or error if it does not exist.
fn get_file(path: &str) -> io::Result<fs::File> {
    return fs::File::open(path);
}

fn read_json_file(path: &str) -> io::Result<String> {
    // Read a JSON file and return it as a string.
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
    for path in PATHS {
        match read_json_file(&format!("{path}/{country}.json")) {
            Ok(j) => return serde_json::from_str(&j).unwrap(),
            Err(_) => continue,
        }
    }

    panic!("bleh");
}

// Get the country folder.
pub fn get_country_folder() -> ReadDir {
    let mut folder_opt = None;
    for path in PATHS {
        match fs::read_dir(format!("{path}/")) {
            Ok(r) => {
                folder_opt = Some(r);
                break;
            }
            Err(_) => continue
        }
    }

    return folder_opt.unwrap();
}

// Function for listing all JSON files in the names folder.
// Used for generating countries in the database.
pub fn get_countries_from_name_files(folder: ReadDir) -> Vec<String> {
    let mut countries = Vec::new();

    for entry in folder {
        let filename = format!("{}", entry.unwrap().file_name().display());
        if !filename.ends_with(".json") { continue }

        let country_name = String::from(&filename[0..filename.len() - 5]);
        countries.push(country_name);
    }

    return countries;
}