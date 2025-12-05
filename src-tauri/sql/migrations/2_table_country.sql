CREATE TABLE Country (
    id INTEGER PRIMARY KEY NOT NULL CHECK (id > 0),
    country_name TEXT NOT NULL,
    names TEXT NOT NULL,    -- NamePool structs
    flag_path TEXT NOT NULL
) STRICT;