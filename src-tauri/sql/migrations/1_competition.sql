CREATE TABLE IF NOT EXISTS Competition (
    id INTEGER PRIMARY KEY,
    comp_name TEXT NOT NULL,
    season_window TEXT NOT NULL, -- AnnualWindow struct
    min_no_of_teams INTEGER NOT NULL,
    format TEXT,    -- Option<Format> struct
    rank_criteria TEXT NOT NULL, -- Vec<RankCriteria> struct
    comp_type TEXT NOT NULL,    -- competition::Type enum
    -- Must have the option to be NULL to indicate that there is no parent.
    parent_id INTEGER,

    FOREIGN KEY (parent_id) REFERENCES Competition(id)
);

CREATE TABLE IF NOT EXISTS CompConnection (
    origin_id INTEGER NOT NULL,
    destination_id INTEGER NOT NULL,
    highest_position INTEGER NOT NULL,
    lowest_position INTEGER NOT NULL,
    team_seeds TEXT NOT NULL,    -- Seed enum
    stats_carry_over INTEGER NOT NULL,   -- bool

    PRIMARY KEY (origin_id, destination_id),
    FOREIGN KEY (origin_id) REFERENCES Competition(id),
    FOREIGN KEY (destination_id) REFERENCES Competition(id)
);

CREATE TABLE IF NOT EXISTS Season (
    id INTEGER PRIMARY KEY,
    comp_id INTEGER NOT NULL,
    season_name TEXT NOT NULL,
    start_date TEXT NOT NULL,    -- Date
    end_date TEXT NOT NULL,  -- Date
    round_robin TEXT,   -- Option<RoundRobin> struct
    knockout_round TEXT,    -- Option<KnockoutRound> struct
    is_over INTEGER NOT NULL,    -- boolean

    FOREIGN KEY (comp_id) REFERENCES Competition(id)
);

 -- Team's data about a given season.
CREATE TABLE IF NOT EXISTS TeamSeason (
    team_id INTEGER NOT NULL,
    season_id INTEGER NOT NULL,
    seed INTEGER NOT NULL,
    rank INTEGER NOT NULL,
    regular_wins INTEGER NOT NULL,
    ot_wins INTEGER NOT NULL,
    draws INTEGER NOT NULL,
    ot_losses INTEGER NOT NULL,
    regular_losses INTEGER NOT NULL,
    goals_scored INTEGER NOT NULL,
    goals_conceded INTEGER NOT NULL,

    PRIMARY KEY (team_id, season_id),
    FOREIGN KEY (team_id) REFERENCES Team(id),
    FOREIGN KEY (season_id) REFERENCES Season(id)
);