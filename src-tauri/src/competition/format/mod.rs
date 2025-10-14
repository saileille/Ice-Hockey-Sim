pub mod round_robin;
pub mod knockout;

use serde_json::json;

use crate::{
    competition::format, match_event
};

#[derive(Debug, serde::Serialize)]
#[derive(Default, Clone, PartialEq)]
enum Type {
    #[default]
    Null,
    RoundRobin,
    Knockout,
}

#[derive(Debug, serde::Serialize)]
#[derive(Default, Clone)]
pub struct Format {
    pub match_rules: match_event::Rules,
    pub round_robin: Option<format::round_robin::RoundRobin>,
    pub knockout: Option<format::knockout::KnockoutRound>,
    format_type: Type,   // Easy way to check whether the competition is a knockout or round robin type.

    // Tests.
    // pub failures: usize,
}

// Basics
impl Format {
    // Build a Format element.
    pub fn build(round_robin: Option<format::round_robin::RoundRobin>, knockout: Option<format::knockout::KnockoutRound>, match_rules: match_event::Rules) -> Option<Self> {
        let mut format = Self::default();
        format.round_robin = round_robin;
        format.knockout = knockout;
        format.match_rules = match_rules;

        // Set the stage type. Only one of round_robin and knockout can be defined.
        if format.round_robin.is_some() {
            if format.knockout.is_none() { format.format_type = Type::RoundRobin }
        }
        else if format.knockout.is_some() { format.format_type = Type::Knockout }

        if format.format_type == Type::Null { return None; }

        return Some(format);
    }

    // Make sure Stage does not have illegal values.
    fn is_valid(&self) -> bool {
        self.match_rules.is_valid() &&
        self.format_type != Type::Null
    }

    // Get JSON for a competition screen.
    pub fn get_comp_screen_json(&self) -> serde_json::Value {
        json!({
            "round_robin": self.round_robin,
            "knockout": self.knockout,
            "match_rules": self.match_rules,
            "type": self.format_type
        })
    }

}