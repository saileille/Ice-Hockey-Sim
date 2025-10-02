pub mod rules;

use self::rules::StageRules;

#[derive(Default)]
pub struct Stage {
    id: usize,
    name: String,
    teams: Vec<TeamData>,
    rules: StageRules,
}

#[derive(Default)]
struct TeamData {
    team_id: usize,
    // rest later
}