mod commands;
mod competition;
mod country;
mod database;
mod event;
mod match_event;
mod io;
mod person;
mod team;


use std::ops::Range;
use std::time::Instant;

use competition::stage::{Stage, TeamData, rules};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    database::initialise();


    // console testing...
    #[cfg(dev)] {
        // Testing with various team counts and match amounts.
        let mut counter: u16 = 0;
        let benchmark: Instant = Instant::now();
        let mut log_str: String = String::from("Teams\tExp Mat\tMatches\tSkipped\tFails");
        for team_count in 10..=20 {
            let mut teams: Vec<usize> = Vec::new();
            for id in (Range {start: 0, end: team_count}) {
                teams.push(id);
            }

            for match_count in (Range {start: 1, end: team_count * 2 - 2}) {
                let mut stage: Stage = Stage::build(
                    "blbl",
                    teams.clone(),
                    rules::RoundRobin::build(0, match_count as u8),
                );

                // Let's skip what is doomed to fail.
                if !stage.has_valid_match_amount() { continue; }
                
                stage.matches_for_round_robin();
                      
                let actual_matches: u8 = stage.get_matches_per_team();
                let skipped: u8 = (match_count as u8) - actual_matches;
                log_str += &format!("\n{}\t{}\t{}\t{}\t{}", team_count, match_count, actual_matches, skipped, stage.failures);
                counter += 1;
            }
        }
        log_str += &format!("\nCreated {} match schedules in {} seconds.", counter, benchmark.elapsed().as_secs_f64());
        println!("{log_str}");
    }

    // console testing over, do as you like.


    tauri::Builder::default()
        .setup(|app| {
            #[cfg(debug_assertions)] {
                let window: tauri::WebviewWindow = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![commands::tests::test_game])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
