//use rand::distr::Uniform;
//use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::database;
use crate::person;

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Team {
    id: usize,
    pub name: String,
    roster: Vec<usize>,
    //lineup: LineUp<'a>,
}

pub fn create_team(name: String) -> Team {
    // Create a team and store it in the database. Return a clone of the Team.
    let team_id: usize = database::TEAMS.lock().unwrap().len() + 1;
    let team: Team = Team {
        id: team_id.clone(),
        name: name,
        roster: Vec::new(),
        //lineup: LineUp::new(),
    };

    database::TEAMS.lock().unwrap().insert(team_id, team.clone());
    return team;
}

impl Team {
    /* fn build_lineup(&'a mut self) {
        // Build a lineup for the team from its roster.
        /*events.sort_by(
            |a, b| 
            a.goal.event.time.get_game_total_seconds(self.rules.period_length)
            .cmp(&b.goal.event.time.get_game_total_seconds(self.rules.period_length))
        );*/

        self.roster.sort_by(|a, b| b.ability.cmp(&a.ability));
        let mut players = Vec::new();
        for player in self.roster.iter() {
            // lineup.auto_add(player);
            players.push(player);
        }

        self.lineup.auto_add(players);

    } */
}

impl Team { // Functions for the testing phase.
    /*pub fn generate_roster(&mut self, min_ability: u8, max_ability: u8) {
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
    }*/
}

pub struct LineUp<'a> {
    // A line-up of players used in a match.
    goalkeepers: [Option<&'a person::Player<'a>>; 2],
    defence_pairs: [DefencePair<'a>; 4],
    forward_lines: [ForwardLine<'a>; 4],
}

impl Default for LineUp<'_> {
    fn default() -> Self {
        LineUp {
            goalkeepers: [
                None,
                None,
            ],
            defence_pairs: [
                DefencePair::new(),
                DefencePair::new(),
                DefencePair::new(),
                DefencePair::new(),
            ],
            forward_lines: [
                ForwardLine::new(),
                ForwardLine::new(),
                ForwardLine::new(),
                ForwardLine::new(),
            ]
        }
    }
}

impl LineUp<'_> {
    pub fn new() -> Self {
        Default::default()
    }
}

/* impl<'a> LineUp<'a> {   // Testing functions.
    fn auto_add<'b>(&'b mut self, players: Vec<&'a person::Player>) {
        // Add players from a roster to the lineup.
        for player in players {
            self.auto_add_player(player);
        }
    }

    fn auto_add_player(&'a mut self, player: &'a person::Player) {
        // Add a player to the lineup.
        match player.position.id {
            person::PositionId::Goalkeeper => self.auto_add_gk(player),
            person::PositionId::Defender => self.auto_add_d(player),
            person::PositionId::LeftWinger => self.auto_add_lw(player),
            person::PositionId::Centre => self.auto_add_c(player),
            person::PositionId::RightWinger => self.auto_add_rw(player),
            person::PositionId::Unknown => return
        }
    }

    fn auto_add_gk(&mut self, player: &'a person::Player) {
        // Add a goalkeeper to the lineup.
        for slot in self.goalkeepers.iter_mut() {
            if slot.is_none() {
                *slot = Some(player);
                return
            }
        }
    }

    fn auto_add_d(&mut self, player: &'a person::Player) {
        // Add a defender to the lineup.
        for pair in self.defence_pairs.iter_mut() {
            if pair.left_defender.is_none() {
                pair.left_defender = Some(player);
                return
            }
            else if pair.right_defender.is_none() {
                pair.right_defender = Some(player);
                return
            }
        }
    }

    fn auto_add_lw(&mut self, player: &'a person::Player) {
        // Add a left winger to the lineup.
        for line in self.forward_lines.iter_mut() {
            if line.left_winger.is_none() {
                line.left_winger = Some(player);
                return
            }
        }
    }

    fn auto_add_c(&mut self, player: &'a person::Player) {
        // Add a centre to the lineup.
        for line in self.forward_lines.iter_mut() {
            if line.centre.is_none() {
                line.centre = Some(player);
                return
            }
        }
    }

    fn auto_add_rw(&mut self, player: &'a person::Player) {
        // Add a right winger to the lineup.
        for line in self.forward_lines.iter_mut() {
            if line.right_winger.is_none() {
                line.right_winger = Some(player);
                return
            }
        }
    }
} */

struct DefencePair<'a> {
    // A pair of defenders used in a line-up.
    left_defender: Option<&'a person::Player<'a>>,
    right_defender: Option<&'a person::Player<'a>>,
}

impl Default for DefencePair<'_> {
    fn default() -> Self {
        DefencePair {
            left_defender: None,
            right_defender: None,
        }
    }
}

impl DefencePair<'_> {
    fn new() -> Self {
        Default::default()
    }
}

struct ForwardLine<'a> {
    // A line of forwards used in a line-up.
    left_winger: Option<&'a person::Player<'a>>,
    centre: Option<&'a person::Player<'a>>,
    right_winger: Option<&'a person::Player<'a>>,
}

impl Default for ForwardLine<'_> {
    fn default() -> Self {
        ForwardLine {
            left_winger: None,
            centre: None,
            right_winger: None,
        }
    }
}

impl ForwardLine<'_> {
    fn new() -> Self {
        Default::default()
    }
}