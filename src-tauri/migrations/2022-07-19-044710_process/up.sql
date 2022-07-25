-- Your SQL goes here
CREATE TABLE process (
    id INTEGER NOT NULL,
    name VARCHAR NOT NULL,
    time INTEGER NOT NULL,
    size INTEGER NOT NULL,
    state INTEGER NOT NULL,
    PRIMARY KEY (id)
)