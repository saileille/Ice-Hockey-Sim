use time::Date;

use crate::logic::{person::{Gender, contract::ContractRole, player::position::PositionId}, types::{AttributeValue, CountryId, PersonId, TeamId}};

struct Person {
    id: PersonId,
    forename: String,
    surname: String,
    gender: Gender,
    country_id: CountryId,
    birthday: Date,
    is_active: bool,

    // calculated
    full_name: Option<String>,
}

struct Player {
    person_id: PersonId,
    ability: AttributeValue,
    position_id: PositionId,
}

struct Manager {
    person_id: PersonId,
    is_human: bool,
}

struct Contract {
    person_id: PersonId,
    team_id: TeamId,
    begin_date: Date,
    end_date: Date,
    role: ContractRole,
    is_signed: bool,
}

struct Position {
    id: PositionId,
    abbreviation: String,
    offensive_value: u8,
}