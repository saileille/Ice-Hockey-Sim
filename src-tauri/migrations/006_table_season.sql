CREATE TABLE Season (
    id INTEGER PRIMARY KEY NOT NULL CHECK (id > 0),
    comp_id INTEGER NOT NULL,
    season_name TEXT NOT NULL,
    start_date TEXT NOT NULL,    -- Date
    end_date TEXT NOT NULL,  -- Date
    round_robin TEXT,   -- Option<RoundRobin> struct
    is_over INTEGER NOT NULL DEFAULT FALSE, -- boolean

    -- Knockout round stuff.
    ko_round_no INTEGER,

    FOREIGN KEY (comp_id) REFERENCES Competition(id) ON DELETE CASCADE
) STRICT;

CREATE TABLE KnockoutPair (
    id INTEGER PRIMARY KEY NOT NULL CHECK (id > 0),
    season_id INTEGER NOT NULL,
    home_id INTEGER NOT NULL,
    away_id INTEGER NOT NULL,
    is_over INTEGER NOT NULL DEFAULT FALSE, -- boolean

    FOREIGN KEY (home_id, season_id) REFERENCES TeamSeason(team_id, season_id) ON DELETE CASCADE,
    FOREIGN KEY (away_id, season_id) REFERENCES TeamSeason(team_id, season_id) ON DELETE CASCADE
) STRICT;

-- Data stored for a team in a knockout pair.
CREATE TABLE KnockoutTeam (
    pair_id INTEGER NOT NULL,
    team_id INTEGER NOT NULL,
    has_advanced INTEGER,   -- A boolean, NULL value indicates an undecided pair.

    -- Mostly same stuff as in TeamSeason, maybe should think of ways to combine this info.
    regular_wins INTEGER NOT NULL DEFAULT 0,
    ot_wins INTEGER NOT NULL DEFAULT 0,
    draws INTEGER NOT NULL DEFAULT 0,
    ot_losses INTEGER NOT NULL DEFAULT 0,
    regular_losses INTEGER NOT NULL DEFAULT 0,
    goals_scored INTEGER NOT NULL DEFAULT 0,
    goals_conceded INTEGER NOT NULL DEFAULT 0,

    all_wins INTEGER AS (regular_wins + ot_wins) STORED,
    all_losses INTEGER AS (ot_losses + regular_losses) STORED,
    games INTEGER AS (all_wins + draws + all_losses) STORED,
    goal_difference INTEGER AS (goals_scored - goals_conceded) STORED,

    PRIMARY KEY (pair_id, team_id),
    FOREIGN KEY (pair_id) REFERENCES KnockoutPair(id) ON DELETE CASCADE,
    FOREIGN KEY (team_id) REFERENCES Team(id) ON DELETE CASCADE
) STRICT;

 -- Team's data about a given season.
CREATE TABLE TeamSeason (
    team_id INTEGER NOT NULL,
    season_id INTEGER NOT NULL,
    seed INTEGER NOT NULL,  -- Can be used in playoffs to indicate regular season position, to denote the previous season's final ranking, etc.
    ranking INTEGER NOT NULL DEFAULT 1,
    regular_wins INTEGER NOT NULL DEFAULT 0,
    ot_wins INTEGER NOT NULL DEFAULT 0,
    draws INTEGER NOT NULL DEFAULT 0,
    ot_losses INTEGER NOT NULL DEFAULT 0,
    regular_losses INTEGER NOT NULL DEFAULT 0,
    goals_scored INTEGER NOT NULL DEFAULT 0,
    goals_conceded INTEGER NOT NULL DEFAULT 0,

    all_wins INTEGER AS (regular_wins + ot_wins) STORED,
    all_losses INTEGER AS (ot_losses + regular_losses) STORED,
    games INTEGER AS (all_wins + draws + all_losses) STORED,
    goal_difference INTEGER AS (goals_scored - goals_conceded) STORED,

    PRIMARY KEY (team_id, season_id),
    FOREIGN KEY (team_id) REFERENCES Team(id) ON DELETE CASCADE,
    FOREIGN KEY (season_id) REFERENCES Season(id) ON DELETE CASCADE
) STRICT;