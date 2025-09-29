use lazy_static::lazy_static;
use std::collections::HashMap;

enum Gender {
    Unknown,    // Stands for null value.
    Male,
    Female,
}

struct Person {
    forename: String,
    surname: String,
    gender: Gender,
}

fn build_person(forename: String, surname: String, gender: Gender) -> Person {
    Person {
        forename: forename,
        surname: surname,
        gender: gender,
    }
}

impl Person {
    fn get_full_name(&self) -> String {
        format!("{} {}", self.forename, self.surname)
    }

    fn get_initial_and_surname(&self) -> String {
        format!("{}. {}", self.forename.chars().nth(0).unwrap(), self.surname)
    }
}

pub struct Player<'a> {
    person: Person,
    ability: u8,
    position: &'a Position,
}

pub fn build_player(forename: String, surname: String, ability: u8, position: &Position) -> Player<'_> {
    Player {
        person: build_person(forename, surname, Gender::Male),
        ability: ability,
        position: position,
    }
}

#[derive(PartialEq, Eq, Hash)]
pub enum PositionId {
    Unknown,    // Stands for null value.
    Goalkeeper,
    Defender,
    LeftWinger,
    Centre,
    RightWinger,
}

pub struct Position {
    id: PositionId,
}

lazy_static! {
    pub static ref POSITIONS: HashMap<PositionId, Position> = {
         let p = HashMap::from([
            (PositionId::Goalkeeper, Position {
                id: PositionId::Goalkeeper
            }),
            (PositionId::Defender, Position {
                id: PositionId::Defender,
            }),
            (PositionId::LeftWinger, Position {
                id: PositionId::LeftWinger,
            }),
            (PositionId::Centre, Position {
                id: PositionId::Centre,
            }),
            (PositionId::RightWinger, Position {
                id: PositionId::RightWinger,
            }),
        ]);
        p
    };
}