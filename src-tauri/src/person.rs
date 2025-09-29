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

fn build_player(forename: String, surname: String, ability: u8, position: &Position) -> Player {
    Player {
        person: build_person(forename, surname, Gender::Male),
        ability: ability,
        position: position,
    }
}

enum PositionId {
    Unknown,    // Basically a null value.
    Goalkeeper,
    Defender,
    LeftWinger,
    Centre,
    RightWinger,
}

struct Position {
    id: PositionId,
}

fn build_position(id: PositionId) -> Position {
    Position {
        id: id,
    }
}