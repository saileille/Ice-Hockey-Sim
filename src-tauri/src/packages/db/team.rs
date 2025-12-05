use crate::logic::{team::lineup::LineUp, types::{CompetitionId, TeamId}};

struct Team {
    id: TeamId,
    full_name: String,
    lineup: LineUp,
    primary_comp_id: CompetitionId,
    actions_remaining: u8,
}