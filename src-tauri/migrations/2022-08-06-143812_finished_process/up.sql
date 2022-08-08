-- Your SQL goes here
CREATE TABLE finished_process (
    id          INTEGER NOT NULL PRIMARY KEY,
    process_id  INTEGER NOT NULL,
    partition_number  INTEGER NOT NULL,
    FOREIGN KEY (process_id) REFERENCES process(id)
)
