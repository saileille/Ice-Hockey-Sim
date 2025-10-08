// Time-related operations.
use time::{
    format_description::{BorrowedFormatItem, parse},
    macros::format_description,
    Date
};
use crate::database::TODAY;

// Use this format for formatting and parsing dates.
static DB_DATE_FORMAT: &[BorrowedFormatItem<'_>] = format_description!("[year]-[month]-[day]");

// Get a Date object for the next time specific day and month will occur.
pub fn get_next_annual_date(month: u8, day: u8) -> Date {
    let today = TODAY.lock().unwrap();

    match Date::parse(&format!("{}-{month:0>2}-{day:0>2}", today.year()), &DB_DATE_FORMAT) {
        Ok(d) => {
            if d < *today {
                return d.replace_year(d.year() + 1).unwrap();
            }
            d
        },
        Err(e) => panic!("{e} - month: {month}, day: {day}")
    }
}

// Get a Date object for the previous time specific day and month occurred.
pub fn get_previous_annual_date(month: u8, day: u8) -> Date {
    let today = TODAY.lock().unwrap();

    match Date::parse(&format!("{}-{month:0>2}-{day:0>2}", today.year()), &DB_DATE_FORMAT) {
        Ok(d) => {
            if d > *today {
                return d.replace_year(d.year() - 1).unwrap();
            }
            d
        },
        Err(e) => panic!("{e} - month: {month}, day: {day}")
    }
}

// Get a vector of dates between two Date objects, inclusive.
pub fn get_dates(start: &Date, end: &Date) -> Vec<Date> {
    let mut dates: Vec<Date> = Vec::new();
    let mut date: Date = start.clone();
    while date <= *end {
        dates.push(date);
        date = date.next_day().unwrap();
    }

    return dates;
}

// Convert a Date object to database string.
pub fn date_to_db_string(date: &Date) -> String {
    date.format(&DB_DATE_FORMAT).unwrap()
}

pub fn db_string_to_date(date: &str) -> Date {
    Date::parse(date, DB_DATE_FORMAT).unwrap()
}