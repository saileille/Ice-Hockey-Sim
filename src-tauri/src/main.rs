// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let _ = fix_path_env::fix(); // Fix path environments for Linux and iOS builds.
    ice_hockey_sim_lib::run()
}
