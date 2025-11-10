use rand::{rngs::ThreadRng, Rng};

use crate::database::POSITIONS;

#[derive(Eq, Hash, PartialEq)]
#[derive(Default, Clone, Debug)]
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
    pub fn get_random(rng: &mut ThreadRng) -> Self {
        let weights = vec![
            (Self::Goalkeeper, 2),
            (Self::LeftDefender, 4),
            (Self::RightDefender, 4),
            (Self::LeftWinger, 4),
            (Self::Centre, 4),
            (Self::RightWinger, 4)
        ];

        let total_weight: u8 = weights.iter().map(|(_, a)| a).sum();
        let random = rng.random_range(0..total_weight);

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
pub struct Position {
    pub id: PositionId,
    pub abbreviation: String,
    pub offensive_value: u8,
}

impl Position {
    pub fn build(id: PositionId, abbreviation: &str, offensive_value: u8) -> Self {
        Self {
            id: id,
            abbreviation: abbreviation.to_string(),
            offensive_value: offensive_value,
        }
    }

    pub fn fetch_from_db(id: &PositionId) -> Self {
        POSITIONS.get(id).expect(&format!("no Position with id {id:#?}")).clone()
    }

    // Make sure the position does not contain illegal values.
    fn is_valid(&self) -> bool {
        self.id != PositionId::default()
    }

    // Get the defensive value of the position as absence of offence.
    fn get_defensive_value(&self) -> u8 {
        u8::MAX - self.offensive_value
    }
}