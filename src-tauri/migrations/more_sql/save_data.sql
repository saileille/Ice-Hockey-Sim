-- All non-static data in the database, condensed to one database cell.
SELECT json_object(
	'CompConnection', (SELECT json_group_array(json_object(
		'origin_id', origin_id,
		'destination_id', destination_id,
		'highest_position', highest_position,
		'lowest_position', lowest_position,
		'team_seeds', team_seeds,
		'stats_carry_over', stats_carry_over
	)) FROM CompConnection),
	'Competition', (SELECT json_group_array(json_object(
		'id', id,
		'comp_name', comp_name,
		'season_window', season_window,
		'min_no_of_teams', min_no_of_teams,
		'rank_criteria', rank_criteria,
		'comp_type', comp_type,
		'parent_id', parent_id,
		'rr_format_id', rr_format_id,
		'kr_format_id', kr_format_id,
		'game_rules_id', game_rules_id
	)) FROM Competition),
	'Contract', (SELECT json_group_array(json_object(
		'person_id', person_id,
		'team_id', team_id,
		'begin_date', begin_date,
		'end_date', end_date,
		'role', role,
		'is_signed', is_signed
	)) FROM Contract),
	'Country', (SELECT json_group_array(json_object(
		'id', id,
		'country_name', country_name,
		'names', names,
		'flag_path', flag_path
	)) FROM Country),
	'Game', (SELECT json_group_array(json_object(
		'id', id,
		'date', date,
		'clock', clock,
		'home_id', home_id,
		'away_id', away_id,
		'season_id', season_id
	)) FROM Game),
	'GameEvent', (SELECT json_group_array(json_object(
		'id', id,
		'game_id', game_id,
		'target_team_id', target_team_id,
		'opponent_team_id', opponent_team_id,
		'time', time,
		'target_players', target_players,
		'opponent_players', opponent_players
	)) FROM GameEvent),
	'GameRules', (SELECT json_group_array(json_object(
		'id', id,
		'periods', periods,
		'period_length', period_length,
		'overtime_length', overtime_length,
		'continuous_overtime', continuous_overtime
	)) FROM GameRules),
	'KnockoutPair', (SELECT json_group_array(json_object(
		'id', id,
		'season_id', season_id,
		'home_id', home_id,
		'away_id', away_id,
		'is_over', is_over
	)) FROM KnockoutPair),
	'KnockoutRoundFormat', (SELECT json_group_array(json_object(
		'id', id,
		'wins_required', wins_required,
		'maximum_games', maximum_games
	)) FROM KnockoutRoundFormat),
	'KnockoutTeam', (SELECT json_group_array(json_object(
		'pair_id', pair_id,
		'team_id', team_id,
		'has_advanced', has_advanced,
		'regular_wins', regular_wins,
		'ot_wins', ot_wins,
		'draws', draws,
		'ot_losses', ot_losses,
		'regular_losses', regular_losses,
		'goals_scored', goals_scored,
		'goals_conceded', goals_conceded
	)) FROM KnockoutTeam),
	'Manager', (SELECT json_group_array(json_object(
		'person_id', person_id,
		'is_human', is_human
	)) FROM Manager),
	'Manager', (SELECT json_group_array(json_object(
		'person_id', person_id,
		'is_human', is_human
	)) FROM Manager),
	'Person', (SELECT json_group_array(json_object(
		'id', id,
		'forename', forename,
		'surname', surname,
		'gender', gender,
		'country_id', country_id,
		'birthday', birthday,
		'is_active', is_active
	)) FROM Person),
	'Player', (SELECT json_group_array(json_object(
		'person_id', person_id,
		'ability', ability,
		'position_id', position_id
	)) FROM Player),
	'RoundRobinFormat', (SELECT json_group_array(json_object(
		'id', id,
		'rounds', rounds,
		'extra_matches', extra_matches,
		'points_for_win', points_for_win,
		'points_for_ot_win', points_for_ot_win,
		'points_for_draw', points_for_draw,
		'points_for_ot_loss', points_for_ot_loss,
		'points_for_loss', points_for_loss
	)) FROM RoundRobinFormat),
	'Season', (SELECT json_group_array(json_object(
		'id', id,
		'comp_id', comp_id,
		'season_name', season_name,
		'start_date', start_date,
		'end_date', end_date,
		'round_robin', round_robin,
		'is_over', is_over,
		'ko_round_no', ko_round_no
	)) FROM Season),
	'ShotEvent', (SELECT json_group_array(json_object(
		'event_id', event_id,
		'shooter_id', shooter_id,
		'assister_1_id', assister_1_id,
		'assister_2_id', assister_2_id,
		'is_goal', is_goal
	)) FROM ShotEvent),
	'Team', (SELECT json_group_array(json_object(
		'id', id,
		'full_name', full_name,
		'lineup', lineup,
		'primary_comp_id', primary_comp_id,
		'player_needs', player_needs,
		'actions_remaining', actions_remaining
	)) FROM Team),
	'TeamGame', (SELECT json_group_array(json_object(
		'game_id', game_id,
		'team_id', team_id,
		'lineup', lineup
	)) FROM TeamGame),
	'TeamSeason', (SELECT json_group_array(json_object(
		'team_id', team_id,
		'season_id', season_id,
		'seed', seed,
		'ranking', ranking,
		'regular_wins', regular_wins,
		'ot_wins', ot_wins,
		'draws', draws,
		'ot_losses', ot_losses,
		'regular_losses', regular_losses,
		'goals_scored', goals_scored,
		'goals_conceded', goals_conceded
	)) FROM TeamSeason),
	'MiscData', $1	-- Miscellaneous data stored in the runtime.
) as d;