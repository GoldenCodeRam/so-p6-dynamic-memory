table! {
    compaction_log (id) {
        id -> Integer,
        iteration -> Integer,
        partition -> Integer,
        previous_position -> Integer,
        final_position -> Integer,
    }
}

table! {
    condensation_log (id) {
        id -> Integer,
        partition -> Integer,
        partition_size -> Integer,
        new_partition -> Integer,
        new_partition_size -> Integer,
    }
}

table! {
    configuration (setting_id) {
        setting_id -> Integer,
        setting_value -> Text,
    }
}

table! {
    finished_process (id) {
        id -> Integer,
        process_id -> Integer,
        partition_number -> Integer,
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
        iteration -> Integer,
        process_id -> Integer,
        storage_partition_id -> Integer,
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
        number -> Integer,
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

joinable!(finished_process -> process (process_id));
joinable!(process_partition -> process (process_id));
joinable!(process_partition -> storage_partition (storage_partition_id));

allow_tables_to_appear_in_same_query!(
    compaction_log,
    condensation_log,
    configuration,
    finished_process,
    iteration_log,
    process,
    process_log,
    process_partition,
    storage_partition,
    storage_partition_log,
);
