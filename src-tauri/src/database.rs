use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

use crate::model::process::Process;
use crate::model::state::StateEnum;
use crate::model::{self};

pub mod models;
pub mod schema;
pub mod statements;

pub fn init_configuration() {
    use crate::model::configuration::SettingName;
    use schema::configuration;

    let connection = establish_connection();

    diesel::delete(configuration::table)
        .execute(&connection)
        .expect("Could not delete table contents");
    diesel::insert_into(configuration::table)
        .values(&models::Configuration {
            setting_id: SettingName::MemorySize as i32,
            setting_value: String::from("50"),
        })
        .execute(&connection)
        .expect("Could not init configuration.");
}

pub fn clear_database() {
    delete_all_iteration_logs();
    delete_all_processes_logs();
    delete_all_processes_partitions();
    delete_all_storage_partitions();
    delete_all_storage_partitions_logs();
}

pub fn add_processes_to_memory() -> bool {
    // Select all ready processes
    println!("Selecting processes with state...");
    let processes = select_processes_with_state(StateEnum::READY as i32);
    let process_partitions = select_all_processes_from_processes_partitions();

    // If there is no ready process in the main list, it means the processor
    // has finished.
    println!("There are {} ready processes.", processes.len());
    if processes.len() == 0 {
        return false;
    } else {
        // If it finishes without adding any process to memory, it means the
        // memory is full or the partitions are not big enough.
        let mut has_added_processes_to_memory = false;

        for process in &processes {
            // If the process is ready but is already on a partition, do not add
            if process_partitions
                .iter()
                .any(|process_already_in_partition| process_already_in_partition.id == process.id)
            {
                continue;
            }
            // First, try to get an empty partition to enter, if there is none,
            // try to create one, if it can't then it means the processor is full
            // of created partitions and it can't enter none. So it has to
            // wait for the processes to end and for a merge to occur.
            let storage_partition = get_empty_storage_partition(process.size)
                .or_else(|| create_storage_partition(process.size));

            if !storage_partition.is_none() {
                insert_process_into_storage_partition(process, &storage_partition.unwrap());
                has_added_processes_to_memory = true;
            }
        }
        return has_added_processes_to_memory;
    }
}

pub fn create_storage_partition_from_remaining_space() {
    use model::configuration::SettingName;

    // Get all the current partitions and calculate the total memory they are
    // using
    let mut used_memory: i32 = 0;
    for partition in select_all_storage_partitions() {
        used_memory += partition.size;
    }

    // Get the remaining space if there is any
    let remaining_space = get_configuration_value(SettingName::MemorySize)
        .setting_value
        .parse::<i32>()
        .unwrap()
        - used_memory;

    if remaining_space > 0 {
        create_storage_partition(remaining_space);
    }
}

pub fn get_empty_storage_partition(process_size: i32) -> Option<models::StoragePartition> {
    use schema::process_partition;
    use schema::storage_partition;

    let connection = establish_connection();

    /*
    Select all the storage partitions with an associated process id, if it has any.
    If it doesn't it means it is empty so a new process can enter.

    +-----------------+    +--------------------+
    |storage_partition|    |process_partition   |
    +-----------------+    +--------------------+
    |id               +--->|process_id          |
    |position         |    |storage_partition_id|
    |size             |    +--------------------+
    +-----------------+
    */
    let storage_partitions = storage_partition::table
        .left_join(process_partition::table)
        .select((
            storage_partition::id,
            storage_partition::position,
            storage_partition::size,
            process_partition::process_id.nullable(),
        ))
        .order(storage_partition::position.asc())
        .load::<(i32, i32, i32, Option<i32>)>(&connection)
        .expect("Could not find storage partitions with processes partitions");

    println!("{:?}", storage_partitions);
    for i in 0..storage_partitions.len() {
        // Then the process might enter here.
        if storage_partitions[i].3.is_none() {
            // Then the process is the same size as the partition, it does not
            // remove or change the partition.
            if storage_partitions[i].2 == process_size {
                return Some(select_storage_partition_with_id(storage_partitions[i].0));
            }
            // It means the partition is bigger than the process, so it has to
            // be removed and changed
            else if storage_partitions[i].2 > process_size {
                // Delete the current partition
                delete_storage_partition_with_id(storage_partitions[i].0);
                // Create a new partition only for the process to fit in
                create_storage_partition_with_position(storage_partitions[i].1, process_size);
                // Next to the created partition create a new one with the remaining space
                create_storage_partition_with_position(
                    storage_partitions[i].1 + 1,
                    storage_partitions[i].2 - process_size,
                );
                // Update all remaining partitions positions
                for e in i + 1..storage_partitions.len() {
                    // All partitons after are moved 1 space to the right
                    update_storage_partition_position(
                        storage_partitions[e].0,
                        storage_partitions[e].1 + 1,
                    );
                }
                // Return the new partiton for the process to fit in
                return Some(statements::select_storage_partition_with_position(
                    storage_partitions[i].1,
                ));
            }
        }
    }
    return None;
}

pub fn merge_storage_partitions() {
    use schema::process_partition;
    use schema::storage_partition;

    let connection = establish_connection();

    // Do al merges until there is no more merging done
    let mut has_finished_merging: bool;
    let mut storage_partitions: Vec<(i32, i32, Option<i32>)>;
    loop {
        // Get all the partitions at the start, so the changes are reflected
        storage_partitions = storage_partition::table
            .left_join(process_partition::table)
            .select((
                storage_partition::id,
                storage_partition::position,
                process_partition::process_id.nullable(),
            ))
            .order(storage_partition::position.asc())
            .load::<(i32, i32, Option<i32>)>(&connection)
            .expect("Could not find free partitions");

        // It assumes the merging has been done here, as if no change is done later
        has_finished_merging = true;
        for i in 0..storage_partitions.len() {
            // If there is no process here, start merging
            if storage_partitions[i].2.is_none() {
                let mut new_partition_size = 0;
                let mut partitions_changed: i32 = 0;

                // Start from the empty partition and see if the next partitions are empty
                for e in i..storage_partitions.len() {
                    if storage_partitions[e].2.is_none() {
                        let storage_partition =
                            select_storage_partition_with_id(storage_partitions[e].0);
                        // Add this partition size to the general partition that may be created
                        // later if the partitions changed is bigger than 1.
                        new_partition_size += storage_partition.size;
                        partitions_changed += 1;
                    } else {
                        break;
                    }
                }

                // If the partitions changed is bigger than 1 it means it found more
                // than 1 adjacent partitions to make the merge.
                if partitions_changed > 1 {
                    // It might be more partitions later to be merged.
                    has_finished_merging = false;
                    /*
                    Start from this partition and delete all the partitions that
                    where empty.
                    Partitions changed = 3,
                     0 1 2 3 4     0 4
                    +-+-+-+-+-+   +-+-+
                    |X|E|E|E|X+-->|X|X|
                    +-+-+-+-+-+   +-+-+
                     */
                    for e in i..i + partitions_changed as usize {
                        delete_storage_partition_with_id(storage_partitions[e].0);
                    }
                    // Create the new big partition from the first position, as if
                    // the partition was big enough from the start
                    create_storage_partition_with_position(i as i32, new_partition_size);
                    // Update all the remaining partition's positions
                    for e in i + partitions_changed as usize..storage_partitions.len() {
                        update_storage_partition_position(
                            storage_partitions[e].0,
                            e as i32 - partitions_changed + 1,
                        );
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

    let iteration_log = statements::select_last_iteration_log();
    let partitions = select_all_storage_partitions();

    for partition in partitions {
        diesel::insert_into(storage_partition_log::table)
            .values(models::NewStoragePartitionLog {
                iteration: iteration_log.id,
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

    let connection = establish_connection();
    let iteration_id = statements::select_last_iteration_log();
    let data: (i32, i32, i32, i32, Option<i32>) = process::table
        .left_join(process_partition::table)
        .select((
            process::id,
            process::state,
            process::time,
            process::size,
            process_partition::storage_partition_id.nullable(),
        ))
        .filter(process::id.eq(process_id))
        .first::<(i32, i32, i32, i32, Option<i32>)>(&connection)
        .expect("Could not load process for logging");

    let log = models::NewProcessLog {
        process_id,
        iteration: iteration_id.id,
        state: data.1,
        storage_partition_id: data.4.unwrap_or(-1),
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

pub fn create_storage_partition_with_position(position: i32, size: i32) {
    use schema::storage_partition;

    let connection = establish_connection();

    diesel::insert_into(storage_partition::table)
        .values(models::NewStoragePartition { position, size })
        .execute(&connection)
        .expect("Could not create storage partition with position.");
}

pub fn create_storage_partition(size: i32) -> Option<models::StoragePartition> {
    use schema::storage_partition;

    if can_create_storage_partition(size) {
        let connection = establish_connection();

        let last_partition_position = storage_partition::table
            .select(storage_partition::position)
            .order(storage_partition::position.desc())
            .first::<i32>(&connection);

        let new_storage_partition: models::NewStoragePartition;
        if last_partition_position.is_err() {
            new_storage_partition = models::NewStoragePartition { position: 0, size };
        } else {
            new_storage_partition = models::NewStoragePartition {
                position: last_partition_position.unwrap() + 1,
                size,
            };
        }

        diesel::insert_into(storage_partition::table)
            .values(new_storage_partition)
            .execute(&connection)
            .expect("Error inserting storage partition.");

        return Some(statements::select_last_storage_partition());
    } else {
        return None;
    }
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

pub fn update_process_with_id(id: i32, process: &Process) -> QueryResult<usize> {
    use schema::process;

    let connection = establish_connection();
    diesel::update(process::table.find(id))
        .set((
            process::name.eq(process.name.to_string()),
            process::time.eq(process.time),
            process::size.eq(process.size),
            process::state.eq(process.state.as_ref().unwrap().get_state_number()),
        ))
        .execute(&connection)
}

pub fn update_storage_partition_position(id: i32, position: i32) {
    use schema::storage_partition;

    let connection = establish_connection();
    println!("new position {}", position);
    diesel::update(storage_partition::table.find(id))
        .set(storage_partition::position.eq(position))
        .execute(&connection)
        .expect("Could not update storage partition position");
}

pub fn select_process_with_id(id: i32) -> QueryResult<models::Process> {
    use schema::process;

    let connection = establish_connection();
    process::table.find(id).first(&connection)
}

pub fn select_storage_partition_with_id(id: i32) -> models::StoragePartition {
    use schema::storage_partition;

    let connection = establish_connection();
    storage_partition::table
        .find(id)
        .first(&connection)
        .expect("Could not find storage partition with id")
}

pub fn select_all_processes_from_processes_partitions() -> Vec<models::Process> {
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
        .expect("Could not select all processes from processes partitions.")
}

pub fn select_all_processes() -> QueryResult<Vec<models::Process>> {
    use schema::process;

    let connection = establish_connection();
    process::table
        .order(process::id)
        .load::<models::Process>(&connection)
}

pub fn select_processes_with_state(state: i32) -> Vec<models::Process> {
    use schema::process;

    let connection = establish_connection();
    process::table
        .order(process::id)
        .filter(process::state.eq(state))
        .load::<models::Process>(&connection)
        .expect("Could not retrieve processes with state.")
}

pub fn select_all_process_logs() -> QueryResult<Vec<(String, i32, i32, i32)>> {
    use schema::process;
    use schema::process_log;

    let connection = establish_connection();
    process_log::table
        .inner_join(process::table.on(process::id.eq(process_log::process_id)))
        .select((
            process::name,
            process_log::state,
            process_log::storage_partition_id,
            process_log::time_remaining,
        ))
        .load::<(String, i32, i32, i32)>(&connection)
}

pub fn select_all_storage_partition_logs() -> QueryResult<Vec<models::StoragePartitionLog>> {
    use schema::storage_partition_log;

    let connection = establish_connection();
    storage_partition_log::table.load::<models::StoragePartitionLog>(&connection)
}

pub fn select_all_storage_partitions() -> Vec<models::StoragePartition> {
    use schema::storage_partition;

    let connection = establish_connection();
    storage_partition::table
        .load::<models::StoragePartition>(&connection)
        .expect("Could not get storage partitions.")
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

fn get_configuration_value(value: model::configuration::SettingName) -> models::Configuration {
    use schema::configuration;

    let connection = establish_connection();
    configuration::table
        .find(value as i32)
        .first::<models::Configuration>(&connection)
        .expect("Could not get configuration.")
}

fn can_create_storage_partition(size: i32) -> bool {
    use model::configuration::SettingName;

    let partitions = select_all_storage_partitions();

    let mut used_memory: i32 = 0;
    for partition in partitions {
        used_memory += partition.size;
    }

    return used_memory + size
        <= get_configuration_value(SettingName::MemorySize)
            .setting_value
            .parse::<i32>()
            .unwrap();
}

fn insert_process_into_storage_partition(
    process: &models::Process,
    storage_partition: &models::StoragePartition,
) {
    create_process_partition(models::ProcessPartition {
        process_id: process.id,
        storage_partition_id: storage_partition.id,
    });
}
