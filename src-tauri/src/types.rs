// Custom types that are widely used are defined here.

// Database ID types.
pub type CountryId = u8;
pub type CompetitionId = u8;
pub type GameId = u8;

pub type TeamId = u8;
pub type PlayerId = u16;

// Type conversions.
pub mod convert {
    // Convert usize to u8.
    pub fn usize_to_u8(num: usize) -> u8 {
        match num.try_into() {
            Ok(n) => n,
            Err(e) => panic!("{e}")
        }
    }

    // Convert usize to u16.
    pub fn usize_to_u16(num: usize) -> u16 {
        match num.try_into() {
            Ok(n) => n,
            Err(e) => panic!("{e}")
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

    // Convert u32 to u8.
    pub fn u32_to_u8(num: u32) -> u8 {
        match num.try_into() {
            Ok(n) => n,
            Err(e) => panic!("{e}")
        }
    }

    // Convert u32 to i32.
    pub fn u32_to_i32(num: u32) -> i32 {
        match num.try_into() {
            Ok(n) => n,
            Err(e) => panic!("{e}")
        }
    }

    // Convert u16 to u8.
    pub fn u16_to_u8(num: u16) -> u8 {
        match num.try_into() {
            Ok(n) => n,
            Err(e) => panic!("{e}")
        }
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