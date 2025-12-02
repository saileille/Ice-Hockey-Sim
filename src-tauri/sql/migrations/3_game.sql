CREATE TABLE Game (
    id INTEGER PRIMARY KEY,
    date TEXT NOT NULL,
    clock INTEGER NOT NULL, -- seconds passed in the game.
    home_id INTEGER NOT NULL,
    away_id INTEGER NOT NULL,
    season_id INTEGER NOT NULL,

    FOREIGN KEY (home_id) REFERENCES Team(id) ON DELETE CASCADE,
    FOREIGN KEY (away_id) REFERENCES Team(id) ON DELETE CASCADE,
    FOREIGN KEY (season_id) REFERENCES Season(id) ON DELETE CASCADE,
    FOREIGN KEY (home_id, season_id) REFERENCES TeamSeason(team_id, season_id) ON DELETE CASCADE,
    FOREIGN KEY (away_id, season_id) REFERENCES TeamSeason(team_id, season_id) ON DELETE CASCADE
);

 -- The match data of a team.
CREATE TABLE TeamGame (
    game_id INTEGER NOT NULL,
    team_id INTEGER NOT NULL,
    lineup TEXT NOT NULL,   -- LineUp struct

    PRIMARY KEY (game_id, team_id),
    FOREIGN KEY (game_id) REFERENCES Game(id) ON DELETE CASCADE,
    FOREIGN KEY (team_id) REFERENCES Team(id) ON DELETE CASCADE
);

CREATE TABLE GameEvent (
    id INTEGER PRIMARY KEY,
    game_id INTEGER NOT NULL,
    target_team_id INTEGER NOT NULL,
    opponent_team_id INTEGER NOT NULL,
    time INTEGER NOT NULL,  -- game time in seconds
    target_players TEXT NOT NULL,    -- PlayersOnIce struct
    opponent_players TEXT NOT NULL,    -- PlayersOnIce struct

    FOREIGN KEY (game_id) REFERENCES Game(id) ON DELETE CASCADE,
    FOREIGN KEY (target_team_id) REFERENCES Team(id) ON DELETE CASCADE,
    FOREIGN KEY (opponent_team_id) REFERENCES Team(id) ON DELETE CASCADE,
    FOREIGN KEY (game_id, target_team_id) REFERENCES TeamGame(game_id, team_id) ON DELETE CASCADE,
    FOREIGN KEY (game_id, opponent_team_id) REFERENCES TeamGame(game_id, team_id) ON DELETE CASCADE
);

CREATE TABLE ShotEvent (
    event_id INTEGER PRIMARY KEY,
    shooter_id INTEGER NOT NULL,
    assister_1_id INTEGER,
    assister_2_id INTEGER,
    is_goal INTEGER NOT NULL,   -- boolean

    FOREIGN KEY (event_id) REFERENCES GameEvent(id) ON DELETE CASCADE,
    FOREIGN KEY (shooter_id) REFERENCES Player(person_id) ON DELETE CASCADE,
    FOREIGN KEY (shooter_id) REFERENCES Person(id) ON DELETE CASCADE,

    FOREIGN KEY (assister_1_id) REFERENCES Player(person_id)
    ON DELETE SET NULL ON UPDATE CASCADE,

    FOREIGN KEY (assister_1_id) REFERENCES Person(id)
    ON DELETE SET NULL ON UPDATE CASCADE,

    FOREIGN KEY (assister_2_id) REFERENCES Player(person_id)
    ON DELETE SET NULL ON UPDATE CASCADE,

    FOREIGN KEY (assister_2_id) REFERENCES Person(id)
    ON DELETE SET NULL ON UPDATE CASCADE
);