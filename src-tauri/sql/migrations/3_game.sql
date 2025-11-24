CREATE TABLE IF NOT EXISTS Game (
    id INTEGER PRIMARY KEY,
    date TEXT NOT NULL,
    home_id INTEGER NOT NULL,
    away_id INTEGER NOT NULL,
    clock TEXT NOT NULL, -- GameClock struct
    season_id INTEGER NOT NULL,

    FOREIGN KEY (home_id) REFERENCES Team(id),
    FOREIGN KEY (away_id) REFERENCES Team(id),
    FOREIGN KEY (season_id) REFERENCES Season(id)
);

 -- The match data of a team.
CREATE TABLE IF NOT EXISTS TeamGame (
    game_id INTEGER NOT NULL,
    team_id INTEGER NOT NULL,
    shots TEXT NOT NULL, -- Vec<Shot> struct
    lineup TEXT NOT NULL,    -- LineUp struct

    PRIMARY KEY (game_id, team_id),
    FOREIGN KEY (game_id) REFERENCES Game(id),
    FOREIGN KEY (team_id) REFERENCES Team(id)
);