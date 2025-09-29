use rand::distr::Uniform;
use rand::Rng;

use crate::person;

pub struct Team<'a> {
    pub name: String,
    roster: Vec<person::Player<'a>>,
}

pub fn build_team<'a>(name: String) -> Team<'a> {
    Team {
        name: name,
        roster: Vec::new(),
    }
}

impl Team<'_> {


    // Test stuffs.

    pub fn generate_roster(&mut self, min_ability: u8, max_ability: u8) {
        /*
         Generate a roster of players for the team.
        */
        self.roster = Vec::new();

        let range = Uniform::new_inclusive(min_ability, max_ability).unwrap();
        let mut rng = rand::rng();

        // Goalkeepers...
        for _ in 0..1 {
            self.roster.push(
                person::build_player(
                    String::from("Placeholder"),
                    String::from("Name"),
                    rng.sample(range),
                    person::POSITIONS.get(&person::PositionId::Goalkeeper).unwrap()
                )
            );
        }

        // Defenders...
        for _ in 0..8 {
            self.roster.push(
                person::build_player(
                    String::from("Placeholder"),
                    String::from("Name"),
                    rng.sample(range),
                    person::POSITIONS.get(&person::PositionId::Defender).unwrap()
                )
            );
        }

        // Left Wingers...
        for _ in 0..4 {
            self.roster.push(
                person::build_player(
                    String::from("Placeholder"),
                    String::from("Name"),
                    rng.sample(range),
                    person::POSITIONS.get(&person::PositionId::LeftWinger).unwrap()
                )
            );
        }

        // Centres...
        for _ in 0..4 {
            self.roster.push(
                person::build_player(
                    String::from("Placeholder"),
                    String::from("Name"),
                    rng.sample(range),
                    person::POSITIONS.get(&person::PositionId::Centre).unwrap()
                )
            );
        }

        // Right Wingers...
        for _ in 0..4 {
            self.roster.push(
                person::build_player(
                    String::from("Placeholder"),
                    String::from("Name"),
                    rng.sample(range),
                    person::POSITIONS.get(&person::PositionId::RightWinger).unwrap()
                )
            );
        }
    }
}