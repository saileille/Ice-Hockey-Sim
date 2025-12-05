use crate::logic::{competition::{Seed, Type, season::ranking::RankCriteria}, time::AnnualWindow, types::{CompetitionId, GameRulesId, KnockoutRoundFormatId, RoundRobinFormatId}};

struct Competition {
    id: CompetitionId,
    comp_name: String,
    season_window: AnnualWindow,
    min_no_of_teams: u8,
    rank_criteria: Vec<RankCriteria>,
    comp_type: Type,
    parent_id: CompetitionId,   // 0 indicates no parent.
    rr_format_id: RoundRobinFormatId,
    kr_format_id: KnockoutRoundFormatId,
    game_rules_id: GameRulesId,
}

struct GameRules {
    id: GameRulesId,
    periods: u8,
    period_length: u16,
    overtime_length: u16,
    continuous_overtime: bool,

    // calculated
    total_length: Option<u16>,
}

struct RoundRobinFormat {
    id: RoundRobinFormatId,
    rounds: u8,
    extra_matches: u8,
    points_for_win: u8,
    points_for_ot_win: u8,
    points_for_draw: u8,
    points_for_ot_loss: u8,
    points_for_loss: u8,
}

struct KnockoutRoundFormat {
    id: KnockoutRoundFormatId,
    wins_required: u8,

    // calculated
    maximum_games: Option<u8>,
}

struct CompConnection {
    origin_id: CompetitionId,
    destination_id: CompetitionId,
    highest_position: u8,
    lowest_position: u8,
    team_seeds: Seed,
    stats_carry_over: bool,
}