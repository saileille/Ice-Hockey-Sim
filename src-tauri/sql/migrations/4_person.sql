CREATE TABLE IF NOT EXISTS Person (
    id INTEGER PRIMARY KEY,
    forename TEXT NOT NULL,
    surname TEXT NOT NULL,
    gender TEXT NOT NULL,    -- Gender enum
    country_id INTEGER NOT NULL,
    birthday TEXT NOT NULL,
    is_active INTEGER NOT NULL,  -- boolean

    FOREIGN KEY (country_id) REFERENCES Country(id)
);

CREATE TABLE IF NOT EXISTS Player (
    person_id INTEGER PRIMARY KEY,
    ability INTEGER NOT NULL, -- the value of PlayerAttribute struct
    position_id INTEGER NOT NULL, -- PositionId enum

    FOREIGN KEY (person_id) REFERENCES Person(id) ON DELETE CASCADE,
    FOREIGN KEY (position_id) REFERENCES Position(id)
);

CREATE TABLE IF NOT EXISTS Manager (
    person_id INTEGER PRIMARY KEY,
    is_human INTEGER NOT NULL,   -- boolean

    FOREIGN KEY (person_id) REFERENCES Person(id) ON DELETE CASCADE
);

 -- Contracts and contract offers.
CREATE TABLE IF NOT EXISTS Contract (
    person_id INTEGER NOT NULL,
    team_id INTEGER NOT NULL,
    begin_date TEXT NOT NULL,
    end_date TEXT NOT NULL,
    role TEXT NOT NULL, -- ContractRole enum
    is_signed INTEGER NOT NULL,  -- Boolean. Difference between a contract and a contract offer.

    PRIMARY KEY (person_id, team_id),
    FOREIGN KEY (person_id) REFERENCES Person(id) ON DELETE CASCADE,
    FOREIGN KEY (team_id) REFERENCES Team(id)
);

CREATE TABLE IF NOT EXISTS Position (
    id INTEGER PRIMARY KEY,    -- PositionId enum
    abbreviation TEXT UNIQUE NOT NULL,
    offensive_value INTEGER NOT NULL
);