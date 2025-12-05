// Custom types that are widely used are defined here.
use std::collections::HashMap;
use sqlx::SqlitePool;

use crate::logic::{country::NamePool, person::Gender};

pub type Db = SqlitePool;

// Database ID types.
pub type CountryId = u8;
pub type RoundRobinFormatId = u8;
pub type KnockoutRoundFormatId = u8;
pub type GameRulesId = u8;
pub type CompetitionId = u8;
pub type SeasonId = u8;
pub type KnockoutPairId = u8;
pub type GameId = u16;
pub type GameEventId = u16;

pub type TeamId = u8;
pub type PersonId = u16;

pub type CountryNamePool = HashMap<Gender, HashMap<String, NamePool>>;
pub type GameSeconds = u16;  // Seconds elapsed in a match. 2 bytes is enough for over 18 hours.

// In-code attribute values. The display value is calculated separately.
pub type AttributeValue = u16;
pub type AttributeDisplayValue = u8;

// Type conversions.
pub mod convert {
    use std::fmt::Display;

    // Convert between integers.
    pub fn int<N1: Display + Copy, N2: TryFrom<N1>>(num: N1) -> N2 {
        match num.try_into() {
            Ok(n) => n,
            Err(_) => panic!("num: {num}")
        }
    }

    // Convert usize to f64.
    pub fn usize_to_f64(num: usize) -> f64 {
        if num <= (f64::MAX as usize) {
            return num as f64;
        }
        panic!("{num} is bigger than {}", f64::MAX);
    }
}