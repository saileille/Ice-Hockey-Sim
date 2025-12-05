-- Data that is global, simple and same for everyone, like the in-game date.
CREATE TABLE KeyValue (
    key_name TEXT PRIMARY KEY NOT NULL,
    value_data ANY NOT NULL
) STRICT;