use crate::person;

pub struct Team<'a> {
    pub name: String,
    roster: Vec<person::Player<'a>>,
}

pub fn build_team(name: String) -> Team<'static> {
    Team {
        name: name,
        roster: Vec::new(),
    }
}