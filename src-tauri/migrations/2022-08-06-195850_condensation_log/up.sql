-- Your SQL goes here
CREATE TABLE condensation_log (
    id INTEGER NOT NULL PRIMARY KEY,
    partition INTEGER NOT NULL,
    partition_size INTEGER NOT NULL,
    new_partition INTEGER NOT NULL,
    new_partition_size INTEGER NOT NULL
)
