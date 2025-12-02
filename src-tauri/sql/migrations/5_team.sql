CREATE TABLE Team (
    id INTEGER PRIMARY KEY,
    full_name TEXT UNIQUE NOT NULL,
    lineup TEXT NOT NULL,   -- LineUp struct
    primary_comp_id INTEGER,
    player_needs TEXT NOT NULL, -- Vec<PlayerNeed> struct
    actions_remaining INTEGER NOT NULL,

    FOREIGN KEY (primary_comp_id) REFERENCES Competition(id)
    ON DELETE SET NULL ON UPDATE CASCADE
);