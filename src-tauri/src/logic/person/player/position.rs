use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Eq, Hash, PartialEq)]
#[derive(Default, Copy, Clone, Debug)]
#[derive(Serialize, Deserialize)]
#[derive(sqlx::Type)]
#[repr(u8)]
pub enum PositionId {
    #[default]
    Null = 0,
    Goalkeeper = 1,
    LeftDefender = 2,
    RightDefender = 3,
    LeftWinger = 4,
    Centre = 5,
    RightWinger = 6,
}

impl PositionId {
    // Get a random position, weighted by need.
    pub fn get_random() -> Self {
        let weights = vec![
            (Self::Goalkeeper, 2),
            (Self::LeftDefender, 4),
            (Self::RightDefender, 4),
            (Self::LeftWinger, 4),
            (Self::Centre, 4),
            (Self::RightWinger, 4)
        ];

        let total_weight: u8 = weights.iter().map(|(_, a)| a).sum();
        let random = rand::random_range(0..total_weight);

        let mut counter = 0;
        for (id, weight) in weights {
            counter += weight;
            if random < counter {
                return id;
            }
        }

        return Self::Null;
    }
}

#[derive(Default, Clone)]
#[derive(FromRow)]
pub struct _Position {
    pub id: PositionId,
    pub abbreviation: String,
    pub offensive_value: u8,
}