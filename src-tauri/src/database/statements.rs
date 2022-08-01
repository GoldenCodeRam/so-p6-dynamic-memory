use diesel::prelude::*;

use crate::model::configuration::SettingName;

use super::{establish_connection, models, schema};

pub fn select_last_storage_partition() -> models::StoragePartition {
    use schema::storage_partition;

    let connection = establish_connection();
    storage_partition::table
        .order(storage_partition::position.desc())
        .first::<models::StoragePartition>(&connection)
        .expect("Could not find last storage partition")
}

pub fn select_last_iteration_log() -> models::IterationLog {
    use schema::iteration_log;

    let connection = establish_connection();
    iteration_log::table
        .order(iteration_log::id.desc())
        .first::<models::IterationLog>(&connection)
        .expect("Could not find last iteration log")
}

pub fn select_storage_partition_with_position(position: i32) -> models::StoragePartition {
    use schema::storage_partition;

    let connection = establish_connection();
    storage_partition::table
        .filter(storage_partition::position.eq(position))
        .first::<models::StoragePartition>(&connection)
        .expect("Could not find partition with position")
}

pub fn update_memory_size(size: i32) {
    use schema::configuration;

    let connection = establish_connection();
    diesel::update(configuration::table.find(SettingName::MemorySize as i32))
        .set(configuration::setting_value.eq(size.to_string()))
        .execute(&connection)
        .expect("Could not update memory size");
}
