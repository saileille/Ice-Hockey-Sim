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

    // Tests.
    // pub failures: usize,
}

// Basics
impl Format {
    // Build a Format element.
    pub fn build(round_robin: Option<RoundRobinFormat>, knockout_round: Option<KnockoutRoundFormat>, match_rules: MatchRules) -> Option<Self> {
        let mut format = Self::default();
        format.round_robin = round_robin;
        format.knockout_round = knockout_round;
        format.match_rules = match_rules;

        // Set the stage type. Only one of round_robin and knockout can be defined.
        if format.round_robin.is_some() {
            if format.knockout_round.is_none() { format.format_type = Type::RoundRobin }
        }
        else if format.knockout_round.is_some() { format.format_type = Type::Knockout }

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
            "knockout": self.knockout_round,
            "match_rules": self.match_rules,
            "type": self.format_type
        })
    }

}