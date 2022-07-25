-- Your SQL goes here
CREATE TABLE process_partition (
    process_id INTEGER NOT NULL,
    storage_partition_id INTEGER NOT NULL,
    FOREIGN KEY (process_id) REFERENCES process(id),
    FOREIGN KEY (storage_partition_id) REFERENCES storage_partition(id),
    PRIMARY KEY (process_id, storage_partition_id)
)