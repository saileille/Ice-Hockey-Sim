// Input/output logic.
use std::{
    collections::HashMap,
    fs,
    io,
    io::Read
};

// Get a file, or error if it does not exist.
fn get_file<S: AsRef<str>>(path: S) -> io::Result<fs::File> {
    return fs::File::open(path.as_ref());
}

fn read_json_file<S: AsRef<str>>(path: S) -> io::Result<String> {
    // Read a JSON file and return it as a string.
    let mut json: String = String::new();
    // let file_result: Result<File, io::Error> = File::open(path.as_ref());
    let mut file: fs::File = match fs::File::open(path.as_ref()) {
        Ok(f) => f,
        Err(e) => return Err(e),
    };

    file.read_to_string(&mut json);
    return Ok(json);
}

pub fn load_country_names<S: AsRef<str>>(country: S) -> HashMap<String, HashMap<String, u16>> {
    // Load names of a specific country.
    let json: String = match read_json_file(format!("./json/names/{}.json", country.as_ref())) {
        Ok(j) => j,
        Err(_) => {
            read_json_file(format!("E:/Tiedostot/koodaus/Tauri/icehockeysim/Ice Hockey Sim/src-tauri/json/names/{}.json", country.as_ref())).unwrap()
        }
    };

    let names: HashMap<String, HashMap<String, u16>> = serde_json::from_str(&json).unwrap();
    return names
}

// Function for listing all JSON files in the names folder.
// Used for generating countries in the database.
pub fn get_countries_from_name_files() -> Vec<String> {
    let paths: fs::ReadDir = match fs::read_dir("./json/names/") {
        Ok(r) => r,
        Err(_) => fs::read_dir("E:/Tiedostot/koodaus/Tauri/icehockeysim/Ice Hockey Sim/src-tauri/json/names/").unwrap(),
    };

    let mut countries: Vec<String> = Vec::new();

    for path in paths {
        let filename: String = format!("{}", path.unwrap().file_name().display());
        if !filename.ends_with(".json") { continue }

        let country_name: String = String::from(&filename[0..filename.len() - 5]);
        countries.push(country_name);
    }

    return countries;
}