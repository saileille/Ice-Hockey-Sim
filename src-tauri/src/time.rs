// Time-related operations.
use std::fmt::Debug;

use time::{
    format_description::BorrowedFormatItem, macros::format_description, Date
};
use crate::database::TODAY;

// Use this format for formatting and parsing dates.
static DB_DATE_FORMAT: &[BorrowedFormatItem<'_>] = format_description!("[year]-[month]-[day]");

// A struct that represents an annual time period with fixed start and end dates. Both start and end are given as [month, day].
#[derive(Debug, serde::Serialize)]
#[derive(Default, Clone)]
pub struct AnnualWindow {
    start: AnnualDate,
    pub end: AnnualDate,
}

impl AnnualWindow {
    // Build the object.
    pub fn build(start: AnnualDate, end: AnnualDate) -> Self {
        AnnualWindow { start: start, end: end }
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

        self.start.day == today.day() &&
        self.start.month == today.month() as u8
    }

    // Check if the current date is the last day of the window.
    fn is_last_day(&self) -> bool {
        let today = TODAY.lock().unwrap();

        self.end.day == today.day() &&
        self.end.month == today.month() as u8
    }

    // Get the previous earliest match date as a date object.
    fn get_previous_start_date(&self) -> Date {
        self.start.get_previous_annual_date()
    }

    // Get the previous latest match date as a date object.
    fn get_previous_end_date(&self) -> Date {
        self.end.get_previous_annual_date()
    }

    // Get the next earliest match date as a date object.
    pub fn get_next_start_date(&self) -> Date {
        self.start.get_next_annual_date()
    }

    // Get the next latest match date as a date object.
    pub fn get_next_end_date(&self) -> Date {
        self.end.get_next_annual_date()
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
        let start_date = self.start.get_date(year);
        let mut end_date = self.end.get_date(year);

        if end_date < start_date {
            end_date = end_date.replace_year(end_date.year() + 1).unwrap();
        }

        return (start_date, end_date)
    }

    // Get start and end date from a given end year.
    fn get_dates_from_end_year(&self, year: i32) -> (Date, Date) {
        let mut start_date = self.start.get_date(year);
        let end_date = self.end.get_date(year);

        if start_date > end_date {
            start_date = start_date.replace_year(start_date.year() - 1).unwrap();
        }

        return (start_date, end_date)
    }
}

// Functions for getting dates out of annual date.
#[derive(Debug, serde::Serialize)]
#[derive(Default, Clone)]
pub struct AnnualDate {
    month: u8,
    day: u8,
}

impl AnnualDate {
    pub fn build<M: TryInto<u8>>(month: M, day: u8) -> Self
        where <M as TryInto<u8>>::Error: Debug {
        Self {
            month: month.try_into().unwrap(),
            day: day,
        }
    }

    // Get a Date object.
    fn get_date(&self, year: i32) -> Date {
        match Date::parse(&format!("{}-{:0>2}-{:0>2}", year, self.month, self.day), &DB_DATE_FORMAT) {
            Ok(d) => d,
            Err(e) => panic!("{e} - year: {year}")
        }
    }

    // Get a Date object for the next time specific day and month will occur.
    fn get_next_annual_date(&self) -> Date {
        let today = TODAY.lock().unwrap();
        let date = self.get_date(today.year());

        if date < *today { return date.replace_year(date.year() + 1).unwrap()}
        return date;
    }

    // Get a Date object for the previous time specific day and month occurred.
    fn get_previous_annual_date(&self) -> Date {
        let today = TODAY.lock().unwrap();
        let date = self.get_date(today.year());

        if date > *today {
            return date.replace_year(date.year() - 1).unwrap();
        }

        return date;
    }

    // Get a date this many years into the future or the past.
    // 0 returns the next year it happens.
    pub fn get_next_date_with_year_offset(&self, years: i32) -> Date {
        let date = self.get_next_annual_date();
        return self.get_date(date.year() + years);
    }

    // Get a date this many years into the future or the past.
    // 0 returns the next year it happens.
    pub fn get_previous_date_with_year_offset(&self, years: i32) -> Date {
        let date = self.get_previous_annual_date();
        return self.get_date(date.year() + years);
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

// Convert a Date object to database string.
pub fn date_to_db_string(date: &Date) -> String {
    date.format(&DB_DATE_FORMAT).unwrap()
}

pub fn db_string_to_date(date: &str) -> Date {
    Date::parse(date, DB_DATE_FORMAT).unwrap()
}