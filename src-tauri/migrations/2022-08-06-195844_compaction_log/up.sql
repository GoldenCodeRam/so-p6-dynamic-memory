-- Your SQL goes here
CREATE TABLE compaction_log (
    id INTEGER NOT NULL PRIMARY KEY,
    iteration INTEGER NOT NULL,
    partition INTEGER NOT NULL,
    previous_position INTEGER NOT NULL,
    final_position INTEGER NOT NULL
)

