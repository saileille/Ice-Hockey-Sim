// The game database.
use std::{collections::HashMap, sync::{LazyLock, Mutex}};
use crate::team::Team;

pub static TEAMS: LazyLock<Mutex<HashMap<usize, Team>>> = LazyLock::new(|| Mutex::new(HashMap::new()));