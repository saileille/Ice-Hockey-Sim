use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use sqlx::{Decode, Encode, Sqlite, encode::IsNull, error::BoxDynError, sqlite::{SqliteArgumentValue, SqliteTypeInfo, SqliteValueRef}};
use time::{self, format_description::BorrowedFormatItem, Date, macros::format_description};


static ISO_FORMAT: &[BorrowedFormatItem<'_>] = format_description!("[year]-[month]-[day]");

// A struct that represents an annual time period with fixed start and end dates. Both start and end are given as [month, day].
#[derive(Default)]
#[derive(Serialize, Deserialize)]
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
    fn _is_active(&self, today: Date) -> bool {
        self.next_start_date(today) > self.next_end_date(today)
    }

    // Check if the current date is the first day of the window.
    fn _is_first_day(&self, today: Date) -> bool {
        self.start.day == today.day() &&
        self.start.month == today.month() as u8
    }

    // Check if the current date is the last day of the window.
    pub fn is_last_day(&self, today: Date) -> bool {
        self.end.day == today.day() &&
        self.end.month == today.month() as u8
    }

    // Get the previous earliest match date as a date object.
    fn _previous_start_date(&self, today: Date) -> Date {
        self.start.previous_annual_date(today)
    }

    // Get the previous latest match date as a date object.
    fn _previous_end_date(&self, today: Date) -> Date {
        self.end.previous_annual_date(today)
    }

    // Get the next earliest match date as a date object.
    pub fn next_start_date(&self, today: Date) -> Date {
        self.start.next_annual_date(today)
    }

    // Get the next latest match date as a date object.
    pub fn next_end_date(&self, today: Date) -> Date {
        self.end.next_annual_date(today)
    }

    // Get the start and end dates for this season if ongoing, or the next if over.
    fn _next_or_ongoing_window_boundaries(&self, today: Date) -> (Date, Date) {
        let end = self.next_end_date(today);
        let mut start = self.next_start_date(today);
        if start > end {
            start = self._previous_start_date(today);
        }

        return (start, end);
    }

    // Get start and end date from a given start year.
    pub fn dates_from_start_year(&self, year: i32) -> (Date, Date) {
        let start_date = self.start.date_object(year);
        let mut end_date = self.end.date_object(year);

        if end_date < start_date {
            end_date = end_date.replace_year(end_date.year() + 1).unwrap();
        }

        return (start_date, end_date)
    }

    // Get start and end date from a given end year.
    fn _dates_from_end_year(&self, year: i32) -> (Date, Date) {
        let mut start_date = self.start.date_object(year);
        let end_date = self.end.date_object(year);

        if start_date > end_date {
            start_date = start_date.replace_year(start_date.year() - 1).unwrap();
        }

        return (start_date, end_date)
    }
}

// Functions for getting dates out of annual date.
#[derive(Default)]
#[derive(Serialize, Deserialize)]
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

    fn iso_date_string(&self, year: i32) -> String {
        format!("{}-{:0>2}-{:0>2}", year, self.month, self.day)
    }

    // Get a Date object.
    pub fn date_object(&self, year: i32) -> Date {
        match Date::parse(self.iso_date_string(year).as_str(), ISO_FORMAT) {
            Ok(d) => d,
            Err(e) => panic!("{e} - year: {year}")
        }
    }

    // Get a Date object for the next time specific day and month will occur.
    fn next_annual_date(&self, today: Date) -> Date {
        let date = self.date_object(today.year());
        if date < today { return date.replace_year(date.year() + 1).unwrap()}
        return date;
    }

    // Get a Date object for the previous time specific day and month occurred.
    fn previous_annual_date(&self, today: Date) -> Date {
        let date = self.date_object(today.year());

        if date > today {
            return date.replace_year(date.year() - 1).unwrap();
        }

        return date;
    }

    // Get a date this many years into the future or the past from the next event.
    // 0 returns the next year it happens.
    pub fn _next_date_with_year_offset(&self, today: Date, years: i32) -> Date {
        let date = self.next_annual_date(today);
        return self.date_object(date.year() + years);
    }

    // Get a date this many years into the future or the past since last event.
    // 0 returns the most recent time it happened.
    pub fn previous_date_with_year_offset(&self, today: Date, years: i32) -> Date {
        let date = self.previous_annual_date(today);
        return self.date_object(date.year() + years);
    }
}