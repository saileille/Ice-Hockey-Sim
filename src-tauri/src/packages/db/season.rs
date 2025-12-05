use time::Date;

use crate::logic::{competition::season::round_robin::RoundRobin, types::{CompetitionId, KnockoutPairId, SeasonId, TeamId}};

struct Season {
    id: SeasonId,
    comp_id: CompetitionId,
    season_name: String,
    start_date: Date,
    end_date: Date,
    round_robin: Option<RoundRobin>,
    is_over: bool,

    // Knockout round stuff.
    ko_round_no: u8,
}

struct KnockoutPair {
    id: KnockoutPairId,
    season_id: SeasonId,
    home_id: TeamId,
    away_id: TeamId,
    is_over: bool,
}

struct KnockoutTeam {
    pair_id: KnockoutPairId,
    team_id: TeamId,
    has_advanced: Option<bool>,
    regular_wins: u8,
    ot_wins: u8,
    draws: u8,
    ot_losses: u8,
    regular_losses: u8,
    goals_scored: u16,
    goals_conceded: u16,

    // calculated
    all_wins: Option<u8>,
    all_losses: Option<u8>,
    games: Option<u8>,
    goal_difference: Option<i16>,
}

struct TeamSeason {
    team_id: TeamId,
    season_id: SeasonId,
    seed: u8,
    ranking: u8,
    regular_wins: u8,
    ot_wins: u8,
    draws: u8,
    ot_losses: u8,
    regular_losses: u8,
    goals_scored: u16,
    goals_conceded: u16,

    // calculated
    all_wins: Option<u8>,
    all_losses: Option<u8>,
    games: Option<u8>,
    goal_difference: Option<i16>,
}