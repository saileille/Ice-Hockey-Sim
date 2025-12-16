pub mod competition;
pub mod time;

use sqlx::SqlitePool;

pub type Db = SqlitePool;

pub type CompetitionId = u8;