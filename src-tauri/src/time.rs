// Time-related operations.
use std::time::Duration;
use time::{
    format_description::{BorrowedFormatItem, parse},
    macros::format_description,
    Date
};
use crate::database::TODAY;

// Use this format for formatting and parsing dates.
static DB_DATE_FORMAT: &[BorrowedFormatItem<'_>] = format_description!("[year]-[month]-[day]");
static SECONDS_IN_DAY: u64 = 86400;

// A struct that represents an annual time period with fixed start and end dates. Both start and end are given as [month, day].
#[derive(Debug)]
#[derive(Default, Clone)]
pub struct AnnualWindow {
    start: [u8; 2],
    end: [u8; 2],
}

impl AnnualWindow {
    // Build the object.
    pub fn build(start_month: u8, start_day: u8, end_month: u8, end_day: u8) -> Self {
        AnnualWindow { start: [start_month, start_day], end: [end_month, end_day] }
    }
}

impl AnnualWindow {
    // Check if the current date is between the start and the end date.
    fn is_active(&self) -> bool {
        self.get_next_start_date() > self.get_next_end_date()
    }

    // Check if the current date is the first day of the window.
    fn is_first_day(&self) -> bool {
        let today = TODAY.lock().unwrap();

        self.start[1] == today.day() &&
        self.start[0] == today.month() as u8
    }

    // Check if the current date is the last day of the window.
    fn is_last_day(&self) -> bool {
        let today = TODAY.lock().unwrap();

        self.end[1] == today.day() &&
        self.end[0] == today.month() as u8
    }

    // Get the previous earliest match date as a date object.
    fn get_previous_start_date(&self) -> Date {
        Self::get_previous_annual_date(&self.start)
    }

    // Get the previous latest match date as a date object.
    fn get_previous_end_date(&self) -> Date {
        Self::get_previous_annual_date(&self.end)
    }

    // Get the next earliest match date as a date object.
    pub fn get_next_start_date(&self) -> Date {
        Self::get_next_annual_date(&self.start)
    }

    // Get the next latest match date as a date object.
    pub fn get_next_end_date(&self) -> Date {
        Self::get_next_annual_date(&self.end)
    }

    // Get the start and end dates for this season if ongoing, or the next if over.
    fn get_next_or_ongoing_window_boundaries(&self) -> (Date, Date) {
        let end = self.get_next_end_date();
        let mut start = self.get_next_start_date();
        if start > end {
            start = self.get_previous_start_date();
        }

        return (start, end);
    }

    // Get start and end date from a given start year.
    pub fn get_dates_from_start_year(&self, year: i32) -> (Date, Date) {
        let start_date = match Date::parse(&format!("{}-{:0>2}-{:0>2}", year, self.start[0], self.start[1]), &DB_DATE_FORMAT) {
            Ok(d) => d,
            Err(e) => panic!("{e} - year: {year}")
        };

        let end_date = match Date::parse(&format!("{}-{:0>2}-{:0>2}", year, self.end[0], self.end[1]), &DB_DATE_FORMAT) {
            Ok(d) => {
                if d < start_date {
                    d.replace_year(year + 1).unwrap()
                }
                else {
                    d
                }
            },
            Err(e) => panic!("{e} - year: {year}")
        };

        return (start_date, end_date)
    }

    // Get start and end date from a given end year.
    fn get_dates_from_end_year(&self, year: i32) -> (Date, Date) {
        let end_date = match Date::parse(&format!("{}-{:0>2}-{:0>2}", year, self.end[0], self.end[1]), &DB_DATE_FORMAT) {
            Ok(d) => d,
            Err(e) => panic!("{e} - year: {year}")
        };

        let start_date = match Date::parse(&format!("{}-{:0>2}-{:0>2}", year, self.start[0], self.start[1]), &DB_DATE_FORMAT) {
            Ok(d) => {
                if d > end_date {
                    d.replace_year(year - 1).unwrap()
                }
                else {
                    d
                }
            },
            Err(e) => panic!("{e} - year: {year}")
        };

        return (start_date, end_date)
    }
}

// Statics.
impl AnnualWindow {
    // Get a Date object for the next time specific day and month will occur.
    fn get_next_annual_date(date: &[u8; 2]) -> Date {
        let today = TODAY.lock().unwrap();

        match Date::parse(&format!("{}-{:0>2}-{:0>2}", today.year(), date[0], date[1]), &DB_DATE_FORMAT) {
            Ok(d) => {
                if d < *today {
                    return d.replace_year(d.year() + 1).unwrap();
                }
                d
            },
            Err(e) => panic!("{e} - date: {date:?}")
        }
    }

    // Get a Date object for the previous time specific day and month occurred.
    fn get_previous_annual_date(date: &[u8; 2]) -> Date {
        let today = TODAY.lock().unwrap();

        match Date::parse(&format!("{}-{:0>2}-{:0>2}", today.year(), date[0], date[1]), &DB_DATE_FORMAT) {
            Ok(d) => {
                if d > *today {
                    return d.replace_year(d.year() - 1).unwrap();
                }
                d
            },
            Err(e) => panic!("{e} - date: {date:?}")
        }
    }
}

// Get a vector of dates between two Date objects, inclusive.
pub fn get_dates(start: &Date, end: &Date) -> Vec<Date> {
    let mut dates = Vec::new();
    let mut date = start.clone();
    while date <= *end {
        dates.push(date);
        date = date.next_day().unwrap();
    }

    return dates;
}

// Get std::time::Duration from desired days.
pub fn get_duration_from_days(days: u64) -> Duration {
    Duration::new(days * SECONDS_IN_DAY, 0)
}

// Convert a Date object to database string.
pub fn date_to_db_string(date: &Date) -> String {
    date.format(&DB_DATE_FORMAT).unwrap()
}

pub fn db_string_to_date(date: &str) -> Date {
    Date::parse(date, DB_DATE_FORMAT).unwrap()
}