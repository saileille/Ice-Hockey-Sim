CREATE TABLE Competition (
    id INTEGER PRIMARY KEY NOT NULL CHECK (id > 0),
    comp_name TEXT NOT NULL,
    season_window TEXT NOT NULL, -- AnnualWindow struct
    min_no_of_teams INTEGER NOT NULL,
    rank_criteria TEXT NOT NULL, -- Vec<RankCriteria> struct
    comp_type TEXT NOT NULL,    -- competition::Type enum
    parent_id INTEGER,  -- Must have the option to be NULL to indicate that there is no parent.
    rr_format_id INTEGER,
    kr_format_id INTEGER,
    game_rules_id INTEGER,

    FOREIGN KEY (parent_id) REFERENCES Competition(id)
    ON DELETE SET NULL ON UPDATE CASCADE,

    FOREIGN KEY (rr_format_id) REFERENCES RoundRobinFormat(id)
    ON DELETE SET NULL ON UPDATE CASCADE,

    FOREIGN KEY (kr_format_id) REFERENCES KnockoutRoundFormat(id)
    ON DELETE SET NULL ON UPDATE CASCADE,

    FOREIGN KEY (game_rules_id) REFERENCES GameRules(id)
    ON DELETE SET NULL ON UPDATE CASCADE
) STRICT;

CREATE TABLE GameRules (
    id INTEGER PRIMARY KEY NOT NULL CHECK (id > 0),
    periods INTEGER NOT NULL,
    period_length INTEGER NOT NULL,
    overtime_length INTEGER NOT NULL,
    continuous_overtime INTEGER NOT NULL,   -- boolean

    total_length INTEGER AS (periods * period_length) STORED
) STRICT;

CREATE TABLE RoundRobinFormat (
    id INTEGER PRIMARY KEY NOT NULL CHECK (id > 0),
    rounds INTEGER NOT NULL,
    extra_matches INTEGER NOT NULL,
    points_for_win INTEGER NOT NULL,
    points_for_ot_win INTEGER NOT NULL,
    points_for_draw INTEGER NOT NULL,
    points_for_ot_loss INTEGER NOT NULL,
    points_for_loss INTEGER NOT NULL
) STRICT;

CREATE TABLE KnockoutRoundFormat(
    id INTEGER PRIMARY KEY NOT NULL CHECK (id > 0),
    wins_required INTEGER NOT NULL,

    maximum_games INTEGER AS (wins_required * 2 - 1) STORED
) STRICT;

CREATE TABLE CompConnection (
    origin_id INTEGER NOT NULL,
    destination_id INTEGER NOT NULL,
    highest_position INTEGER NOT NULL,
    lowest_position INTEGER NOT NULL,
    team_seeds TEXT NOT NULL,    -- Seed enum
    stats_carry_over INTEGER NOT NULL,   -- bool

    PRIMARY KEY (origin_id, destination_id),
    FOREIGN KEY (origin_id) REFERENCES Competition(id) ON DELETE CASCADE,
    FOREIGN KEY (destination_id) REFERENCES Competition(id) ON DELETE CASCADE
) STRICT;