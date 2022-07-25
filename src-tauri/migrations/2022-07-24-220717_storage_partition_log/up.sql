-- Your SQL goes here
CREATE TABLE storage_partition_log (
    id INTEGER NOT NULL,
    iteration INTEGER NOT NULL,
    storage_partition_id INTEGER NOT NULL,
    position INTEGER NOT NULL,
    size INTEGER NOT NULL,
    PRIMARY KEY (id)
)