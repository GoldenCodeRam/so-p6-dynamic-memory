-- Your SQL goes here
CREATE TABLE process_log (
    id INTEGER NOT NULL,
    iteration INTEGER NOT NULL,
    process_id INTEGER NOT NULL,
    storage_partition_id INTEGER NOT NULL,
    time_remaining INTEGER NOT NULL,
    state INTEGER NOT NULL,
    PRIMARY KEY (id)
)