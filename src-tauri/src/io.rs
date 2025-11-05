// Input/output logic.
use std::{collections::HashMap, fs, io, io::Read};


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
    let json = match read_json_file(&format!("./json/names/{}.json", country)) {
        Ok(j) => j,
        Err(_) => {
            read_json_file(&format!("E:/Tiedostot/koodaus/Tauri/icehockeysim/Ice Hockey Sim/src-tauri/json/names/{}.json", country)).unwrap()
        }
    };

    return serde_json::from_str(&json).unwrap();
}

// Function for listing all JSON files in the names folder.
// Used for generating countries in the database.
pub fn get_countries_from_name_files() -> Vec<String> {
    let paths = match fs::read_dir("./json/names/") {
        Ok(r) => r,
        Err(_) => fs::read_dir("E:/Tiedostot/koodaus/Tauri/icehockeysim/Ice Hockey Sim/src-tauri/json/names/").unwrap(),
    };

    let mut countries = Vec::new();

    for path in paths {
        let filename = format!("{}", path.unwrap().file_name().display());
        if !filename.ends_with(".json") { continue }

        let country_name = String::from(&filename[0..filename.len() - 5]);
        countries.push(country_name);
    }

    return countries;
}