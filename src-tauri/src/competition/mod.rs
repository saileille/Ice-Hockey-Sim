// Competition data.
pub mod stage;

use self::stage::Stage;

#[derive(Default)]
struct Competition {
    id: usize,
    name: String,
    team_ids: Vec<usize>,
    stages: Vec<Vec<Stage>>,    // nested so that simultaneous stages are easier to mark.
}