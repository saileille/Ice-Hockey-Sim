use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum RankCriteria {
    Seed,   // Lower is better.
    Points,
    GoalDifference,
    GoalsScored,
    GoalsConceded,  // Lower is better.
    RegularWins,
    TotalWins,
    OvertimeWins,
    Draws,
    OvertimeLosses,
    RegularLosses,  // Lower is better.
    TotalLosses,    // Lower is better.

    // Takes rankings from all child competitions, with latest competition having highest priority.
    ChildCompRanking,

    // Usually last resort, although competitions should have the ability to not sort at all.
    Random,
}