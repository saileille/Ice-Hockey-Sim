// Input/output logic.
use std::collections::HashMap;
use std::fs::{create_dir_all, File};
use std::io::{Read, Write};

use crate::team::Team;

fn create_database() {
    let _result = create_database_folder();
    create_json_file("teams");
}

fn create_database_folder() -> std::io::Result<()> {
    // Create the database for the program.
    create_dir_all("/json")?;
    Ok(())
}

fn create_json_file(name: &str) {
    // Create the given JSON file if it does not exist already.
    let Ok(mut file) = File::options().write(true).create_new(true).open(format!("/json/{name}.json")) else { return };
    file.write_all(b"{}").unwrap();
}

fn read_json_file(name: &str) -> String {
    // Read a JSON file and return it as a string.
    let mut json: String = String::new();
    let mut file: File = File::open(format!("/json/{name}.json")).unwrap();
    file.read_to_string(&mut json).unwrap();
    return json
}

fn write_json_file(name: &str, json: String) {
    // Write to a JSON file.
    let mut file: File = File::create(format!("/json/{name}.json")).unwrap();
    file.write_all(json.as_bytes()).unwrap();
}

pub fn load_teams() -> HashMap<usize, Team> {
    // Load teams.json.
    let json: String = read_json_file("teams");
    let teams: HashMap<usize, Team> = serde_json::from_str(&json).unwrap();
    return teams
}

pub fn save_teams(teams: &HashMap<usize, Team>) {
    // Save teams.json.
    let json: String = serde_json::to_string(teams).unwrap();
    write_json_file("teams", json);
}