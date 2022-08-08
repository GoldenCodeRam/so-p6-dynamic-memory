use diesel::prelude::*;

use super::{establish_connection, models, schema};

pub fn create_compaction_log(partition: i32, previous_position: i32, final_position: i32) -> () {
    use schema::compaction_log;

    let connection = establish_connection();
    diesel::insert_into(compaction_log::table)
        .values(models::NewCompactionLog {
            iteration: select_last_iteration_log().id,
            partition,
            previous_position,
            final_position,
        })
        .execute(&connection)
        .expect("Error creating compaction log");
}

pub fn create_condensation_log(
    partition: i32, 
    partition_size: i32,
    new_partition: i32,
    new_partition_size: i32,
) -> () {
    use schema::condensation_log;
    let connection = establish_connection();
    diesel::insert_into(condensation_log::table)
        .values(models::NewCondensationLog {
            partition,
            partition_size,
            new_partition,
            new_partition_size,
        })
        .execute(&connection)
        .expect("Error creating condensation log");
}

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

pub fn select_all_storage_partitions_and_process_partitions() -> Vec<(i32, i32, i32, i32, Option<i32>)> {
    use schema::process_partition;
    use schema::storage_partition;

    let connection = establish_connection();
    return storage_partition::table
        .left_join(process_partition::table)
        .select((
            storage_partition::id,
            storage_partition::position,
            storage_partition::number,
            storage_partition::size,
            process_partition::process_id.nullable(),
        ))
        .order(storage_partition::position.asc())
        .load::<(i32, i32, i32, i32, Option<i32>)>(&connection)
        .expect("Could not find storage partitions and process partitions");
}
