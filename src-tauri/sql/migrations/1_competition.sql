CREATE TABLE Competition (
    id INTEGER PRIMARY KEY,
    comp_name TEXT NOT NULL,
    season_window TEXT NOT NULL, -- AnnualWindow struct
    min_no_of_teams INTEGER NOT NULL,
    rank_criteria TEXT NOT NULL, -- Vec<RankCriteria> struct
    comp_type TEXT NOT NULL,    -- competition::Type enum
    -- Must have the option to be NULL to indicate that there is no parent.
    parent_id INTEGER,

    FOREIGN KEY (parent_id) REFERENCES Competition(id)
    ON DELETE SET NULL ON UPDATE CASCADE
);

CREATE TABLE MatchRules (
    comp_id INTEGER PRIMARY KEY,
    periods INTEGER NOT NULL,
    period_length INTEGER NOT NULL,
    overtime_length INTEGER NOT NULL,
    continuous_overtime INTEGER NOT NULL,   -- boolean

    total_length INTEGER AS (periods * period_length) STORED,

    FOREIGN KEY (comp_id) REFERENCES Competition(id) ON DELETE CASCADE
);

CREATE TABLE RoundRobinFormat (
    comp_id INTEGER PRIMARY KEY,
    rounds INTEGER NOT NULL,
    extra_matches INTEGER NOT NULL,
    points_for_win INTEGER NOT NULL,
    points_for_ot_win INTEGER NOT NULL,
    points_for_draw INTEGER NOT NULL,
    points_for_ot_loss INTEGER NOT NULL,
    points_for_loss INTEGER NOT NULL,

    FOREIGN KEY (comp_id) REFERENCES Competition(id) ON DELETE CASCADE
);

CREATE TABLE KnockoutRoundFormat(
    comp_id INTEGER PRIMARY KEY,
    wins_required INTEGER NOT NULL,

    maximum_games INT AS (wins_required * 2 - 1) STORED,

    FOREIGN KEY (comp_id) REFERENCES Competition(id) ON DELETE CASCADE
);

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
);

CREATE TABLE Season (
    id INTEGER PRIMARY KEY,
    comp_id INTEGER NOT NULL,
    season_name TEXT NOT NULL,
    start_date TEXT NOT NULL,    -- Date
    end_date TEXT NOT NULL,  -- Date
    round_robin TEXT,   -- Option<RoundRobin> struct
    knockout_round TEXT,    -- Option<KnockoutRound> struct
    is_over INTEGER NOT NULL,    -- boolean

    FOREIGN KEY (comp_id) REFERENCES Competition(id) ON DELETE CASCADE
);

 -- Team's data about a given season.
CREATE TABLE TeamSeason (
    team_id INTEGER NOT NULL,
    season_id INTEGER NOT NULL,
    seed INTEGER NOT NULL,
    ranking INTEGER NOT NULL,
    regular_wins INTEGER NOT NULL,
    ot_wins INTEGER NOT NULL,
    draws INTEGER NOT NULL,
    ot_losses INTEGER NOT NULL,
    regular_losses INTEGER NOT NULL,
    goals_scored INTEGER NOT NULL,
    goals_conceded INTEGER NOT NULL,

    all_wins INTEGER AS (regular_wins + ot_wins) STORED,
    all_losses INTEGER AS (ot_losses + regular_losses) STORED,
    games INTEGER AS (all_wins + draws + all_losses) STORED,
    goal_difference INTEGER AS (goals_scored - goals_conceded) STORED,

    PRIMARY KEY (team_id, season_id),
    FOREIGN KEY (team_id) REFERENCES Team(id),
    FOREIGN KEY (season_id) REFERENCES Season(id) ON DELETE CASCADE
);