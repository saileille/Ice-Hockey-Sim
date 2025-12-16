-- Insert data from JSON.

INSERT INTO CompConnection (
    origin_id,
    destination_id,
    highest_position,
    lowest_position,
    team_seeds,
    stats_carry_over
)
SELECT
    json_extract(value, '$.origin_id'),
    json_extract(value, '$.destination_id'),
    json_extract(value, '$.highest_position'),
    json_extract(value, '$.lowest_position'),
    json_extract(value, '$.team_seeds'),
    json_extract(value, '$.stats_carry_over')
FROM json_each(json_extract($1, '$.CompConnection'));

INSERT INTO Competition (
    id,
	comp_name,
	season_window,
	min_no_of_teams,
	rank_criteria,
	comp_type,
	parent_id,
	rr_format_id,
	kr_format_id,
	game_rules_id
)
SELECT
    json_extract(value, '$.id'),
    json_extract(value, '$.comp_name'),
    json_extract(value, '$.season_window'),
    json_extract(value, '$.min_no_of_teams'),
    json_extract(value, '$.rank_criteria'),
    json_extract(value, '$.comp_type'),
    json_extract(value, '$.parent_id'),
    json_extract(value, '$.rr_format_id'),
    json_extract(value, '$.kr_format_id'),
    json_extract(value, '$.game_rules_id')
FROM json_each(json_extract($1, '$.Competition'));

INSERT INTO Contract (
    person_id,
	team_id,
	begin_date,
	end_date,
	role,
	is_signed
)
SELECT
    json_extract(value, '$.person_id'),
    json_extract(value, '$.team_id'),
    json_extract(value, '$.begin_date'),
    json_extract(value, '$.end_date'),
    json_extract(value, '$.role'),
    json_extract(value, '$.is_signed')
FROM json_each(json_extract($1, '$.Contract'));

INSERT INTO Country (
    id,
	country_name,
	names,
	flag_path
)
SELECT
    json_extract(value, '$.id'),
    json_extract(value, '$.country_name'),
    json_extract(value, '$.names'),
    json_extract(value, '$.flag_path')
FROM json_each(json_extract($1, '$.Country'));

INSERT INTO Game (
    id,
	date,
	clock,
	home_id,
	away_id,
	season_id
)
SELECT
    json_extract(value, '$.id'),
    json_extract(value, '$.date'),
    json_extract(value, '$.clock'),
    json_extract(value, '$.home_id'),
    json_extract(value, '$.away_id'),
    json_extract(value, '$.season_id')
FROM json_each(json_extract($1, '$.Game'));

INSERT INTO GameEvent (
    id,
	game_id,
	target_team_id,
	opponent_team_id,
	time,
	target_players,
	opponent_players
)
SELECT
    json_extract(value, '$.id'),
    json_extract(value, '$.game_id'),
    json_extract(value, '$.target_team_id'),
    json_extract(value, '$.opponent_team_id'),
    json_extract(value, '$.time'),
    json_extract(value, '$.target_players'),
    json_extract(value, '$.opponent_players')
FROM json_each(json_extract($1, '$.GameEvent'));

INSERT INTO GameRules (
    id,
	periods,
	period_length,
	overtime_length,
	continuous_overtime
)
SELECT
    json_extract(value, '$.id'),
    json_extract(value, '$.periods'),
    json_extract(value, '$.period_length'),
    json_extract(value, '$.overtime_length'),
    json_extract(value, '$.continuous_overtime')
FROM json_each(json_extract($1, '$.GameRules'));

INSERT INTO KnockoutPair (
    id,
	season_id,
	home_id,
	away_id,
	is_over
)
SELECT
    json_extract(value, '$.id'),
    json_extract(value, '$.season_id'),
    json_extract(value, '$.home_id'),
    json_extract(value, '$.away_id'),
    json_extract(value, '$.is_over')
FROM json_each(json_extract($1, '$.KnockoutPair'));

INSERT INTO KnockoutRoundFormat (
    id,
	wins_required
)
SELECT
    json_extract(value, '$.id'),
    json_extract(value, '$.wins_required')
FROM json_each(json_extract($1, '$.KnockoutRoundFormat'));

INSERT INTO KnockoutTeam (
    pair_id,
	team_id,
	has_advanced,
	regular_wins,
	ot_wins,
	draws,
	ot_losses,
	regular_losses,
	goals_scored,
	goals_conceded
)
SELECT
    json_extract(value, '$.pair_id'),
    json_extract(value, '$.team_id'),
    json_extract(value, '$.has_advanced'),
    json_extract(value, '$.regular_wins'),
    json_extract(value, '$.ot_wins'),
    json_extract(value, '$.draws'),
    json_extract(value, '$.ot_losses'),
    json_extract(value, '$.regular_losses'),
    json_extract(value, '$.goals_scored'),
    json_extract(value, '$.goals_conceded')
FROM json_each(json_extract($1, '$.KnockoutTeam'));

INSERT INTO Manager (
    person_id,
	is_human
)
SELECT
    json_extract(value, '$.person_id'),
    json_extract(value, '$.is_human')
FROM json_each(json_extract($1, '$.Manager'));

INSERT INTO Person (
    id,
	forename,
	surname,
	gender,
	country_id,
	birthday,
	is_active
)
SELECT
    json_extract(value, '$.id'),
    json_extract(value, '$.forename'),
    json_extract(value, '$.surname'),
    json_extract(value, '$.gender'),
    json_extract(value, '$.country_id'),
    json_extract(value, '$.birthday'),
    json_extract(value, '$.is_active')
FROM json_each(json_extract($1, '$.Person'));

INSERT INTO Player (
    person_id,
	ability,
	position_id
)
SELECT
    json_extract(value, '$.person_id'),
    json_extract(value, '$.ability'),
    json_extract(value, '$.position_id')
FROM json_each(json_extract($1, '$.Player'));

INSERT INTO RoundRobinFormat (
    id,
	rounds,
	extra_matches,
	points_for_win,
	points_for_ot_win,
	points_for_draw,
	points_for_ot_loss,
	points_for_loss
)
SELECT
    json_extract(value, '$.id'),
    json_extract(value, '$.rounds'),
    json_extract(value, '$.extra_matches'),
    json_extract(value, '$.points_for_win'),
    json_extract(value, '$.points_for_ot_win'),
    json_extract(value, '$.points_for_draw'),
    json_extract(value, '$.points_for_ot_loss'),
    json_extract(value, '$.points_for_loss')
FROM json_each(json_extract($1, '$.RoundRobinFormat'));

INSERT INTO Season (
    id,
	comp_id,
	season_name,
	start_date,
	end_date,
	round_robin,
	is_over,
	ko_round_no
)
SELECT
    json_extract(value, '$.id'),
    json_extract(value, '$.comp_id'),
    json_extract(value, '$.season_name'),
    json_extract(value, '$.start_date'),
    json_extract(value, '$.end_date'),
    json_extract(value, '$.round_robin'),
    json_extract(value, '$.is_over'),
    json_extract(value, '$.ko_round_no')
FROM json_each(json_extract($1, '$.Season'));

INSERT INTO ShotEvent (
    event_id,
	shooter_id,
	assister_1_id,
	assister_2_id,
	is_goal
)
SELECT
    json_extract(value, '$.event_id'),
    json_extract(value, '$.shooter_id'),
    json_extract(value, '$.assister_1_id'),
    json_extract(value, '$.assister_2_id'),
    json_extract(value, '$.is_goal')
FROM json_each(json_extract($1, '$.ShotEvent'));

INSERT INTO Team (
    id,
	full_name,
	lineup,
	primary_comp_id,
	player_needs,
	actions_remaining
)
SELECT
    json_extract(value, '$.id'),
    json_extract(value, '$.full_name'),
    json_extract(value, '$.lineup'),
    json_extract(value, '$.primary_comp_id'),
    json_extract(value, '$.player_needs'),
    json_extract(value, '$.actions_remaining')
FROM json_each(json_extract($1, '$.Team'));

INSERT INTO TeamGame (
    game_id,
	team_id,
	lineup
)
SELECT
    json_extract(value, '$.game_id'),
    json_extract(value, '$.team_id'),
    json_extract(value, '$.lineup')
FROM json_each(json_extract($1, '$.TeamGame'));

INSERT INTO TeamSeason (
    team_id,
	season_id,
	seed,
	ranking,
	regular_wins,
	ot_wins,
	draws,
	ot_losses,
	regular_losses,
	goals_scored,
	goals_conceded
)
SELECT
    json_extract(value, '$.team_id'),
    json_extract(value, '$.season_id'),
    json_extract(value, '$.seed'),
    json_extract(value, '$.ranking'),
    json_extract(value, '$.regular_wins'),
    json_extract(value, '$.ot_wins'),
    json_extract(value, '$.draws'),
    json_extract(value, '$.ot_losses'),
    json_extract(value, '$.regular_losses'),
    json_extract(value, '$.goals_scored'),
    json_extract(value, '$.goals_conceded')
FROM json_each(json_extract($1, '$.TeamSeason'));

SELECT json_extract($1, '$.MiscData') AS d;