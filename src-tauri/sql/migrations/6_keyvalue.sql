-- Data that is global, simple and same for everyone, like the in-game date.
CREATE TABLE IF NOT EXISTS KeyValue (
    key_name TEXT PRIMARY KEY,
    value_data ANY NOT NULL
);