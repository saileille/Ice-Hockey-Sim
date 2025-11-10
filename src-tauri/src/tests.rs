
// Functions to help with testing.

use time::Date;

use crate::{commands::continue_game::go_to_next_day, time::db_string_to_date};

pub fn simulate_to_day(date: &str, today: &Date) {
    loop {
        if *today > db_string_to_date(date) {
            break;
        }

        go_to_next_day();
    }
}