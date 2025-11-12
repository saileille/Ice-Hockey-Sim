pub mod round_robin;
pub mod knockout_round;

use serde_json::json;

use crate::{
    competition::format::{knockout_round::KnockoutRound as KnockoutRoundFormat, round_robin::RoundRobin as RoundRobinFormat}, match_event::{self, Rules as MatchRules}
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
    pub round_robin: Option<RoundRobinFormat>,
    pub knockout_round: Option<KnockoutRoundFormat>,
    format_type: Type,   // Easy way to check whether the competition is a knockout or round robin type.
}

// Basics
impl Format {
    // Build a Format element.
    pub fn build(round_robin: Option<RoundRobinFormat>, knockout_round: Option<KnockoutRoundFormat>, match_rules: MatchRules) -> Option<Self> {
        let mut format = Self {
            round_robin: round_robin,
            knockout_round: knockout_round,
            match_rules: match_rules,
            ..Default::default()
        };

        // Set the stage type. Only one of round_robin and knockout can be defined.
        if format.round_robin.is_some() {
            if format.knockout_round.is_none() { format.format_type = Type::RoundRobin }
        }
        else if format.knockout_round.is_some() { format.format_type = Type::Knockout }

        if format.format_type == Type::Null { return None; }

        return Some(format);
    }

    // Get JSON for a competition screen.
    pub fn get_comp_screen_package(&self) -> serde_json::Value {
        json!({
            "round_robin": self.round_robin,
            "knockout_round": self.knockout_round,
            "match_rules": self.match_rules,
            "type": self.format_type
        })
    }

}