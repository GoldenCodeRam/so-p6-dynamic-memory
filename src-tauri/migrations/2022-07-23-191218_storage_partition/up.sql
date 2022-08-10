-- Your SQL goes here
CREATE TABLE storage_partition( 
    id INTEGER NOT NULL PRIMARY KEY,
    number INTEGER NOT NULL,
    position INTEGER NOT NULL,
    position_start INTEGER NOT NULL,
    position_end INTEGER NOT NULL,
    size INTEGER NOT NULL
)
