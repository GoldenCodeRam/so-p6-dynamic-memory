use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

use crate::model::process::Process;

pub mod models;
pub mod schema;

pub fn merge_storage_partitions() {
    use schema::process_partition;
    use schema::storage_partition;

    let connection = establish_connection();

    let mut has_finished_merging: bool;
    let mut partition_info: Vec<(i32, Option<i32>, i32)>;
    loop {
        partition_info = storage_partition::table
            .left_join(process_partition::table)
            .select((
                storage_partition::id,
                process_partition::storage_partition_id.nullable(),
                storage_partition::position,
            ))
            .order(storage_partition::position.asc())
            .load::<(i32, Option<i32>, i32)>(&connection)
            .expect("Could not find free partitions");

        println!("{:?}", partition_info);
        has_finished_merging = true;
        for i in 0..partition_info.len() {
            if partition_info[i].1.is_none() {
                let mut new_partition_size = 0;
                let mut partitions_changed: i32 = 0;

                for e in i..partition_info.len() {
                    if partition_info[e].1.is_none() {
                        let storage_partition =
                            select_storage_partition_with_id(partition_info[e].0).unwrap();
                        println!(
                            "p {} | s {}",
                            storage_partition.position, storage_partition.size
                        );
                        new_partition_size += storage_partition.size;
                        partitions_changed += 1;
                    } else {
                        break;
                    }
                }

                if partitions_changed > 1 {
                    has_finished_merging = false;

                    for e in i..i + partitions_changed as usize {
                        delete_storage_partition_with_id(partition_info[e].0);
                    }
                    println!("this partition start {}", i);
                    create_storage_partition_with_position(i as i32, new_partition_size)
                        .expect("Could not create storage partition");

                    for e in i + partitions_changed as usize..partition_info.len() {
                        update_storage_partition_position(
                            partition_info[e].0,
                            e as i32 - partitions_changed + 1,
                        )
                        .expect("Could not update storage partition");
                    }
                    break;
                }
            }
        }

        if has_finished_merging {
            break;
        }
    }
}

pub fn check_process_name_is_unique(process_id: Option<i32>, process_name: &str) -> bool {
    use schema::process;

    let connection = establish_connection();
    if process_id.is_none() {
        process::table
            .filter(schema::process::name.eq(process_name))
            .load::<models::Process>(&connection)
            .expect("Error loading processes")
            .len()
            == 0
    } else {
        process::table
            .filter(schema::process::name.eq(process_name))
            .filter(schema::process::id.ne(process_id.unwrap()))
            .load::<models::Process>(&connection)
            .expect("Error loading processes")
            .len()
            == 0
    }
}

pub fn create_iteration_log() -> QueryResult<models::IterationLog> {
    use schema::iteration_log;

    let connection = establish_connection();
    diesel::insert_into(iteration_log::table)
        .default_values()
        .execute(&connection)
        .expect("Error inserting iteration log");
    iteration_log::table
        .order(iteration_log::id.desc())
        .first::<models::IterationLog>(&connection)
}

pub fn create_storage_partition_logs() {
    use schema::storage_partition_log;

    let connection = establish_connection();

    let iteration_log = select_last_iteration_log().unwrap();
    let partitions = select_all_storage_partitions().unwrap();

    for partition in partitions {
        diesel::insert_into(storage_partition_log::table)
            .values(models::NewStoragePartitionLog {
                iteration: iteration_log.id.unwrap(),
                storage_partition_id: partition.id,
                position: partition.position,
                size: partition.size,
            })
            .execute(&connection)
            .expect("Could not add parititon log");
    }
}

pub fn create_process_log(process_id: i32) {
    use schema::process;
    use schema::process_log;
    use schema::process_partition;
    use schema::storage_partition;

    let connection = establish_connection();
    let iteration_id = select_last_iteration_log().unwrap();
    let data: (i32, i32, i32, i32, Option<i32>, Option<i32>) = process::table
        .left_join(process_partition::table.left_join(storage_partition::table))
        .select((
            process::id,
            process::state,
            process::time,
            process::size,
            storage_partition::id.nullable(),
            storage_partition::size.nullable(),
        ))
        .filter(process::id.eq(process_id))
        .first::<(i32, i32, i32, i32, Option<i32>, Option<i32>)>(&connection)
        .expect("Could not load process for logging");

    let log = models::NewProcessLog {
        process_id,
        iteration_log_id: iteration_id.id.unwrap(),
        state: data.1,
        storage_partition_id: data.4.unwrap_or(-1),
        storage_partition_size: data.5.unwrap_or(-1),
        time_remaining: data.2,
    };

    diesel::insert_into(process_log::table)
        .values(log)
        .execute(&connection)
        .expect("Error inserting process log.");
}

pub fn create_process_partition(process_partition: models::ProcessPartition) {
    use schema::process_partition;

    let connection = establish_connection();
    diesel::insert_into(process_partition::table)
        .values(process_partition)
        .execute(&connection)
        .expect("Error inserting process partition.");
}

pub fn create_storage_partition_with_position(position: i32, size: i32) -> QueryResult<usize> {
    use schema::storage_partition;

    let connection = establish_connection();

    diesel::insert_into(storage_partition::table)
        .values(models::NewStoragePartition { position, size })
        .execute(&connection)
}

pub fn create_storage_partition(size: i32) -> QueryResult<models::StoragePartition> {
    use schema::storage_partition;

    let connection = establish_connection();

    let partition_position = storage_partition::table
        .select(storage_partition::position)
        .order(storage_partition::position.desc())
        .first::<i32>(&connection);

    let mut final_partition_position = partition_position.unwrap_or(-1);
    if final_partition_position == -1 {
        final_partition_position = 0;
    } else {
        final_partition_position += 1;
    }

    let new_partition = models::NewStoragePartition {
        position: final_partition_position,
        size,
    };

    diesel::insert_into(storage_partition::table)
        .values(&new_partition)
        .execute(&connection)
        .expect("Error inserting storage partition.");

    storage_partition::table
        .order(storage_partition::id.desc())
        .first::<models::StoragePartition>(&connection)
}

pub fn delete_process_partition_with_process_id(process_id: i32) {
    use schema::process_partition;

    let connection = establish_connection();
    diesel::delete(process_partition::table)
        .filter(process_partition::process_id.eq(process_id))
        .execute(&connection)
        .expect("Could not delete partition");
}

pub fn delete_storage_partition_with_id(id: i32) {
    use schema::storage_partition;

    let connection = establish_connection();
    diesel::delete(storage_partition::table.filter(storage_partition::id.eq(id)))
        .execute(&connection)
        .expect("Could not delete partition");
}

pub fn create_process(process: Process) -> QueryResult<usize> {
    use schema::process;

    let connection = establish_connection();
    let new_process = models::NewProcess {
        name: process.name.as_str(),
        time: process.time,
        size: process.size,
        state: process.state.unwrap().get_state_number(),
    };

    diesel::insert_into(process::table)
        .values(&new_process)
        .execute(&connection)
}

pub fn update_process_with_id(id: i32, process: Process) -> QueryResult<usize> {
    use schema::process;

    let connection = establish_connection();
    diesel::update(process::table.find(id))
        .set((
            process::name.eq(process.name),
            process::time.eq(process.time),
            process::size.eq(process.size),
            process::state.eq(process.state.unwrap().get_state_number()),
        ))
        .execute(&connection)
}

pub fn update_storage_partition_position(id: i32, position: i32) -> QueryResult<usize> {
    use schema::storage_partition;

    let connection = establish_connection();
    println!("new position {}", position);
    diesel::update(storage_partition::table.find(id))
        .set(storage_partition::position.eq(position))
        .execute(&connection)
}

pub fn select_process_with_id(id: i32) -> QueryResult<models::Process> {
    use schema::process;

    let connection = establish_connection();
    process::table.find(id).first(&connection)
}

pub fn select_last_iteration_log() -> QueryResult<models::IterationLog> {
    use schema::iteration_log;

    let connection = establish_connection();
    iteration_log::table
        .order(iteration_log::id.desc())
        .first::<models::IterationLog>(&connection)
}

pub fn select_storage_partition_with_id(id: i32) -> QueryResult<models::StoragePartition> {
    use schema::storage_partition;

    let connection = establish_connection();
    storage_partition::table.find(id).first(&connection)
}

pub fn select_all_processes_from_processes_partitions() -> QueryResult<Vec<models::Process>> {
    use schema::process;
    use schema::process_partition;
    use schema::storage_partition;

    let connection = establish_connection();
    let processess_ids = process_partition::table
        .inner_join(process::table)
        .inner_join(storage_partition::table)
        .select(process::id)
        .load::<i32>(&connection)
        .expect("Could not find processess ids");

    process::table
        .filter(process::id.eq_any(processess_ids))
        .load::<models::Process>(&connection)
}

pub fn select_all_processes() -> QueryResult<Vec<models::Process>> {
    use schema::process;

    let connection = establish_connection();
    process::table.load::<models::Process>(&connection)
}

pub fn select_all_process_logs() -> QueryResult<Vec<(String, i32, i32, i32, i32)>> {
    use schema::process;
    use schema::process_log;

    let connection = establish_connection();
    process_log::table
        .inner_join(process::table.on(process::id.eq(process_log::process_id)))
        .select((
            process::name,
            process_log::state,
            process_log::iteration_log_id,
            process_log::storage_partition_id,
            process_log::time_remaining
        ))
        .load::<(String, i32, i32, i32, i32)>(&connection)
}

pub fn select_all_storage_partition_logs() -> QueryResult<Vec<models::StoragePartitionLog>> {
    use schema::storage_partition_log;

    let connection = establish_connection();
    storage_partition_log::table.load::<models::StoragePartitionLog>(&connection)
}

pub fn select_all_storage_partitions() -> QueryResult<Vec<models::StoragePartition>> {
    use schema::storage_partition;

    let connection = establish_connection();
    storage_partition::table.load::<models::StoragePartition>(&connection)
}

pub fn delete_all_processes_logs() {
    use schema::process_log;
    let connection = establish_connection();
    diesel::delete(process_log::table)
        .execute(&connection)
        .expect("Could not delete process logs");
}

pub fn delete_all_processes() -> QueryResult<usize> {
    use schema::process;

    let connection = establish_connection();
    diesel::delete(process::table).execute(&connection)
}

pub fn delete_process_with_id(id: i32) -> QueryResult<usize> {
    use schema::process;

    let connection = establish_connection();
    diesel::delete(process::table.filter(schema::process::id.eq(id))).execute(&connection)
}

pub fn delete_all_processes_partitions() -> bool {
    use schema::process_partition;

    let connection = establish_connection();
    diesel::delete(process_partition::table)
        .execute(&connection)
        .is_ok()
}

pub fn delete_all_iteration_logs() {
    use schema::iteration_log;
    let connection = establish_connection();
    diesel::delete(iteration_log::table)
        .execute(&connection)
        .expect("Could not delete iteration logs");
}

pub fn delete_all_storage_partitions() -> bool {
    use schema::storage_partition;

    let connection = establish_connection();
    diesel::delete(storage_partition::table)
        .execute(&connection)
        .is_ok()
}

pub fn delete_all_storage_partitions_logs() -> bool {
    use schema::storage_partition_log;

    let connection = establish_connection();
    diesel::delete(storage_partition_log::table)
        .execute(&connection)
        .is_ok()
}

fn establish_connection() -> SqliteConnection {
    let database_url = "../public/data.sqlite";
    SqliteConnection::establish(database_url)
        .expect(&format!("Error connecting to {}", database_url))
}
