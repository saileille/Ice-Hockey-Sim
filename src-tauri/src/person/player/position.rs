use crate::database::POSITIONS;

#[derive(PartialEq, Eq, Hash)]  // Cannot be used as a HashMap key otherwise.
#[derive(Default, Clone, Debug)]
pub enum PositionId {
    #[default] Null,
    Goalkeeper,
    Defender,
    LeftWinger,
    Centre,
    RightWinger,
}

#[derive(Default, Clone)]
pub struct Position {
    pub id: PositionId,
    pub offensive_value: u8,
}

impl Position {
    pub fn new(id: PositionId, offensive_value: u8) -> Self {
        let mut position: Position = Position::default();
        position.id = id;
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