CREATE TABLE Person (
    id INTEGER PRIMARY KEY NOT NULL CHECK (id > 0),
    forename TEXT NOT NULL,
    surname TEXT NOT NULL,
    gender TEXT NOT NULL,    -- Gender enum
    country_id INTEGER NOT NULL,
    birthday TEXT NOT NULL, -- Date
    is_active INTEGER NOT NULL,  -- boolean

    full_name TEXT AS (forename || ' ' || surname) STORED,

    FOREIGN KEY (country_id) REFERENCES Country(id) ON DELETE CASCADE
) STRICT;

CREATE TABLE Player (
    person_id INTEGER PRIMARY KEY NOT NULL,
    ability INTEGER NOT NULL, -- the value of PlayerAttribute struct
    position_id INTEGER NOT NULL, -- PositionId enum cast to u8

    FOREIGN KEY (person_id) REFERENCES Person(id) ON DELETE CASCADE,
    FOREIGN KEY (position_id) REFERENCES Position(id) ON DELETE CASCADE
) STRICT;

CREATE TABLE Manager (
    person_id INTEGER PRIMARY KEY NOT NULL,
    is_human INTEGER NOT NULL,   -- boolean

    FOREIGN KEY (person_id) REFERENCES Person(id) ON DELETE CASCADE
) STRICT;

 -- Contracts and contract offers.
CREATE TABLE Contract (
    person_id INTEGER NOT NULL,
    team_id INTEGER NOT NULL,
    begin_date TEXT NOT NULL,   -- Date
    end_date TEXT NOT NULL, -- Date
    role TEXT NOT NULL, -- ContractRole enum
    is_signed INTEGER NOT NULL,  -- Boolean. Difference between a contract and a contract offer.

    PRIMARY KEY (person_id, team_id),
    FOREIGN KEY (person_id) REFERENCES Person(id) ON DELETE CASCADE,
    FOREIGN KEY (team_id) REFERENCES Team(id) ON DELETE CASCADE
) STRICT;

CREATE TABLE Position (
    id INTEGER PRIMARY KEY NOT NULL CHECK (id > 0), -- PositionId enum cast to u8
    abbreviation TEXT UNIQUE NOT NULL,
    offensive_value INTEGER NOT NULL
) STRICT;