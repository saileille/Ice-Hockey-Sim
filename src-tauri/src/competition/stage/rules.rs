use crate::match_event;

#[derive(Default)]
pub struct StageRules {
    overtime_length: u16,
    continuous_overtime: bool,
    stage_type: StageType,
    game_rules: match_event::Rules,
    round_robin_rules: RoundRobinRules,
    knockout_rules: KnockoutRules,
}

#[derive(Default)]
struct RoundRobinRules {
    rounds: u8, // Only even numbers supported for now.
    points_for_win: u8,
    points_for_draw: u8,
    points_for_loss: u8,
}

#[derive(Default)]
struct KnockoutRules {
    wins_required: u8,
}

#[derive(Default)]
enum StageType {
    #[default] Null,
    RoundRobin,
    Knockout,
}