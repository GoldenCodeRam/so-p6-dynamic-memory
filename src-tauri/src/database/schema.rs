table! {
    configuration (setting_id) {
        setting_id -> Integer,
        setting_value -> Text,
    }
}

table! {
    iteration_log (id) {
        id -> Integer,
    }
}

table! {
    process (id) {
        id -> Integer,
        name -> Text,
        time -> Integer,
        size -> Integer,
        state -> Integer,
    }
}

table! {
    process_log (id) {
        id -> Integer,
        process_id -> Integer,
        storage_partition_id -> Integer,
        storage_partition_size -> Integer,
        time_remaining -> Integer,
        state -> Integer,
    }
}

table! {
    process_partition (process_id, storage_partition_id) {
        process_id -> Integer,
        storage_partition_id -> Integer,
    }
}

table! {
    storage_partition (id) {
        id -> Integer,
        position -> Integer,
        size -> Integer,
    }
}

table! {
    storage_partition_log (id) {
        id -> Integer,
        iteration -> Integer,
        storage_partition_id -> Integer,
        position -> Integer,
        size -> Integer,
    }
}

joinable!(process_partition -> process (process_id));
joinable!(process_partition -> storage_partition (storage_partition_id));

allow_tables_to_appear_in_same_query!(
    configuration,
    iteration_log,
    process,
    process_log,
    process_partition,
    storage_partition,
    storage_partition_log,
);
