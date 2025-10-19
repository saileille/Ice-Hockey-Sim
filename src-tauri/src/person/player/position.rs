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

#[derive(Default, Clone)]
pub struct Position {
    pub id: PositionId,
    pub abbreviation: String,
    pub offensive_value: u8,
}

impl Position {
    pub fn build(id: PositionId, abbreviation: &str, offensive_value: u8) -> Self {
        let mut position = Position::default();
        position.id = id;
        position.abbreviation = abbreviation.to_string();
        position.offensive_value = offensive_value;
        return position;
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