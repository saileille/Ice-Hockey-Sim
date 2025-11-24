// Time-related operations.
use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use sqlx::{Decode, Encode, Sqlite, encode::IsNull, error::BoxDynError, sqlite::{SqliteArgumentValue, SqliteTypeInfo, SqliteValueRef}};
use time::{
    format_description::BorrowedFormatItem, macros::format_description, Date
};
use crate::{database, types::{Db, convert}};

// Use this format for formatting and parsing dates.
static DB_DATE_FORMAT: &[BorrowedFormatItem<'_>] = format_description!("[year]-[month]-[day]");

// The average amount of days in a year.
static DAYS_IN_YEAR: f64 = 365.2425;

// A struct that represents an annual time period with fixed start and end dates. Both start and end are given as [month, day].
#[derive(Default, Clone)]
#[derive(Debug, Serialize, Deserialize)]
pub struct AnnualWindow {
    start: AnnualDate,
    pub end: AnnualDate,
}

impl sqlx::Type<Sqlite> for AnnualWindow {
    fn type_info() -> SqliteTypeInfo {
        <String as sqlx::Type<Sqlite>>::type_info()
    }
}

impl<'q> Encode<'q, Sqlite> for AnnualWindow {
    fn encode(self, buf: &mut Vec<SqliteArgumentValue<'q>>) -> Result<IsNull, BoxDynError> {
        Encode::<Sqlite>::encode(serde_json::to_string(&self).unwrap(), buf)
    }

    fn encode_by_ref(&self, buf: &mut Vec<SqliteArgumentValue<'q>>) -> Result<IsNull, BoxDynError> {
        Encode::<Sqlite>::encode(serde_json::to_string(self).unwrap(), buf)
    }
}

impl<'r> Decode<'r, Sqlite> for AnnualWindow {
    fn decode(value: SqliteValueRef<'r>) -> Result<Self, BoxDynError> {
        let json = <serde_json::Value as Decode<Sqlite>>::decode(value)?;
        Ok(serde_json::from_value(json)?)
    }
}

impl AnnualWindow {
    // Build the object.
    pub fn build(start: AnnualDate, end: AnnualDate) -> Self {
        AnnualWindow { start: start, end: end }
    }

    // Check if the current date is between the start and the end date.
    async fn _is_active(&self, db: &Db) -> bool {
        self.get_next_start_date(db).await > self.get_next_end_date(db).await
    }

    // Check if the current date is the first day of the window.
    async fn _is_first_day(&self, db: &Db) -> bool {
        let today = database::get_today(db).await;
        self.start.day == today.day() &&
        self.start.month == today.month() as u8
    }

    // Check if the current date is the last day of the window.
    pub async fn is_last_day(&self, db: &Db) -> bool {
        let today = database::get_today(db).await;
        self.end.day == today.day() &&
        self.end.month == today.month() as u8
    }

    // Get the previous earliest match date as a date object.
    async fn _get_previous_start_date(&self, db: &Db) -> Date {
        self.start.get_previous_annual_date(db).await
    }

    // Get the previous latest match date as a date object.
    async fn _get_previous_end_date(&self, db: &Db) -> Date {
        self.end.get_previous_annual_date(db).await
    }

    // Get the next earliest match date as a date object.
    pub async fn get_next_start_date(&self, db: &Db) -> Date {
        self.start.get_next_annual_date(db).await
    }

    // Get the next latest match date as a date object.
    pub async fn get_next_end_date(&self, db: &Db) -> Date {
        self.end.get_next_annual_date(db).await
    }

    // Get the start and end dates for this season if ongoing, or the next if over.
    async fn _get_next_or_ongoing_window_boundaries(&self, db: &Db) -> (Date, Date) {
        let end = self.get_next_end_date(db).await;
        let mut start = self.get_next_start_date(db).await;
        if start > end {
            start = self._get_previous_start_date(db).await;
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
    fn _get_dates_from_end_year(&self, year: i32) -> (Date, Date) {
        let mut start_date = self.start.get_date(year);
        let end_date = self.end.get_date(year);

        if start_date > end_date {
            start_date = start_date.replace_year(start_date.year() - 1).unwrap();
        }

        return (start_date, end_date)
    }
}

// Functions for getting dates out of annual date.
#[derive(Debug, Serialize, Deserialize)]
#[derive(Default, Clone)]
pub struct AnnualDate {
    pub month: u8,
    pub day: u8,
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
    pub fn get_date(&self, year: i32) -> Date {
        match Date::parse(&format!("{}-{:0>2}-{:0>2}", year, self.month, self.day), &DB_DATE_FORMAT) {
            Ok(d) => d,
            Err(e) => panic!("{e} - year: {year}")
        }
    }

    // Get a Date object for the next time specific day and month will occur.
    async fn get_next_annual_date(&self, db: &Db) -> Date {
        let today = database::get_today(db).await;
        let date = self.get_date(today.year());
        if date < today { return date.replace_year(date.year() + 1).unwrap()}
        return date;
    }

    // Get a Date object for the previous time specific day and month occurred.
    async fn get_previous_annual_date(&self, db: &Db) -> Date {
        let today = database::get_today(db).await;
        let date = self.get_date(today.year());

        if date > today {
            return date.replace_year(date.year() - 1).unwrap();
        }

        return date;
    }

    // Get a date this many years into the future or the past from the next event.
    // 0 returns the next year it happens.
    pub async fn _get_next_date_with_year_offset(&self, db: &Db, years: i32) -> Date {
        let date = self.get_next_annual_date(db).await;
        return self.get_date(date.year() + years);
    }

    // Get a date this many years into the future or the past since last event.
    // 0 returns the most recent time it happened.
    pub async fn get_previous_date_with_year_offset(&self, db: &Db, years: i32) -> Date {
        let date = self.get_previous_annual_date(db).await;
        return self.get_date(date.year() + years);
    }
}

// Get a vector of dates between two Date objects, inclusive.
pub fn get_dates(start: Date, end: Date) -> Vec<Date> {
    let mut dates = Vec::new();
    let mut date = start.clone();
    while date <= end {
        dates.push(date);
        date = date.next_day().unwrap();
    }

    return dates;
}

// Convert a Date object to database string.
pub fn date_to_string(date: Date) -> String {
    date.format(&DB_DATE_FORMAT).unwrap()
}

// Convert a database string to Date object.
pub fn _string_to_date(date: &str) -> Date {
    Date::parse(date, DB_DATE_FORMAT).unwrap()
}

// Get how many years there are in-between the two dates.
// Gives positive values if date2 is later than date1.
pub fn get_years_between(date1: Date, date2: Date) -> i8 {
    let years = convert::int::<i32, i8>(date2.year() - date1.year());

    match date2.month() as i8 - date1.month() as i8 {
        1..=i8::MAX => years,
        0 => match date2.day() as i8 - date1.day() as i8 {
            0..=i8::MAX => years,
            _ => years - 1
        },
        _ => years - 1
    }
}

// Convert years to days (roughly).
pub fn years_to_days(years: u8) -> u16 {
    ((years as f64) * DAYS_IN_YEAR) as u16
}