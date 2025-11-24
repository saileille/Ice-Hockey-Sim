use crate::types::Db;

pub struct Directories {
    pub names: String,
    pub flags: String,
}

// For keeping track of stuff.
pub struct AppData {
    pub db: Db,
    pub directories: Directories,
}

impl AppData {
    // Build the thing.
    pub fn build(db: Db, directories: Directories) -> Self {
        Self {
            db,
            directories,
        }
    }
}