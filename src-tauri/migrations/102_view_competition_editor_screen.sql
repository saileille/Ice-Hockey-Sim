CREATE VIEW CompetitionEditorScreen AS SELECT
    id, comp_name, season_window, min_no_of_teams, rank_criteria, comp_type, parent_id,
    periods, period_length, overtime_length, continous_overtime,    -- GameRules
    rounds, extra_matches, points_for_win, points_for_ot_win, points_for_draw, points_for_ot_loss, points_for_loss, -- RoundRobinFormat
    wins_required   -- KnockoutRoundFormat
FROM Competition
LEFT JOIN GameRules ON GameRules.id = Competition.game_rules_id
LEFT JOIN RoundRobinFormat ON RoundRobinFormat.id = Competition.rr_format_id
LEFT JOIN KnockoutRoundFormat ON KnockoutRoundFormat.id = Competition.kr_format_id;