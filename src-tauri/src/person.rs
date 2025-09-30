use lazy_static::lazy_static;
use std::collections::HashMap;

#[derive(Default)]
enum Gender {
    #[default] Unknown,
    Male,
    Female,
}

struct Person {
    forename: String,
    surname: String,
    gender: Gender,
}

impl Default for Person {
    fn default() -> Self {
        Person {
            forename: String::from(""),
            surname: String::from(""),
            gender: Gender::default(),
        }
    }
}

fn build_person(forename: String, surname: String, gender: Gender) -> Person {
    Person {
        forename: forename,
        surname: surname,
        gender: gender,
    }
}

impl Person {
    fn new() -> Self {
        Default::default()
    }


    fn get_full_name(&self) -> String {
        format!("{} {}", self.forename, self.surname)
    }

    fn get_initial_and_surname(&self) -> String {
        format!("{}. {}", self.forename.chars().nth(0).unwrap(), self.surname)
    }
}

pub struct Player<'a> {
    person: Person,
    pub ability: u8,
    pub position: &'a Position,
}

impl Default for Player<'_> {
    fn default() -> Self {
        Player {
            person: Person::default(),
            ability: 0,
            position: POSITIONS.get(&PositionId::Unknown).unwrap(),
        }
    }
}

pub fn build_player(forename: String, surname: String, ability: u8, position: &Position) -> Player<'_> {
    Player {
        person: build_person(forename, surname, Gender::Male),
        ability: ability,
        position: position,
    }
}

impl Player<'_> {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(PartialEq, Eq, Hash)]  // Cannot be used as a HashMap key otherwise.
#[derive(Default)]
pub enum PositionId {
    #[default] Unknown,
    Goalkeeper,
    Defender,
    LeftWinger,
    Centre,
    RightWinger,
}

#[derive(Default)]
pub struct Position {
    pub id: PositionId,
}

impl Position {
    fn new() -> Self {
        Default::default()
    }
}

lazy_static! {
    pub static ref POSITIONS: HashMap<PositionId, Position> = {
         let p = HashMap::from([
            (PositionId::Unknown, Position {
                id: PositionId::Unknown
            }),
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