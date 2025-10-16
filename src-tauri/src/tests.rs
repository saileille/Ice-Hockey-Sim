
// Functions to help with testing.

use crate::{commands::continue_game::go_to_next_day, database::TODAY, time::db_string_to_date};

pub fn simulate_to_day(date: &str) {
    loop {
        let today = TODAY.lock().unwrap().clone();
        if today > db_string_to_date(date) {
            break;
        }

        go_to_next_day();
    }
}