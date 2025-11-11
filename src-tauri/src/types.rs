// Custom types that are widely used are defined here.

use std::collections::HashMap;

use crate::{country::NamePool, person::Gender};

// Database ID types.
pub type CountryId = u8;
pub type CompetitionId = u8;

pub type TeamId = u8;
pub type PlayerId = u16;
pub type ManagerId = u8;

pub type CountryNamePool = HashMap<Gender, HashMap<String, NamePool>>;

// Person attributes. Divide by 100 to get the actual attribute.
pub type AttributeValue = u16;

// Type conversions.
pub mod convert {
    // Convert between integers.
    pub fn int<N1, N2: TryFrom<N1>>(num: N1) -> N2 {
        match num.try_into() {
            Ok(n) => n,
            Err(_) => panic!()
        }
    }

    // Convert usize to f64.
    pub fn usize_to_f64(num: usize) -> f64 {
        if num <= (f64::MAX as usize) {
            return num as f64;
        }
        panic!("{num} is bigger than {}", f64::MAX);
    }

    // Convert f64 to u8.
    pub fn f64_to_u8(num: f64) -> u8 {
        if num <= (u8::MAX as f64) {
            return num as u8;
        }
        panic!("{num} is bigger than {}", u8::MAX);
    }

    // Convert u16 to i16.
    pub fn u16_to_i16(num: u16) -> i16 {
        match num.try_into() {
            Ok(n) => n,
            Err(e) => panic!("{e}")
        }
    }

    // Convert u8 to i8.
    pub fn u8_to_i8(num: u8) -> i8 {
        match num.try_into() {
            Ok(n) => n,
            Err(e) => panic!("{e}")
        }
    }

    // Convert i16 to i8.
    pub fn i16_to_i8(num: i16) -> i8 {
        match num.try_into() {
            Ok(n) => n,
            Err(e) => panic!("{e}")
        }
    }
}