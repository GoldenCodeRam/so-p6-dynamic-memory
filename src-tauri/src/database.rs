use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

use crate::database::statements::select_all_storage_partitions_and_process_partitions;
use crate::model::process::Process;
use crate::model::state::StateEnum;

use self::statements::{create_condensation_log, select_storage_partition_with_position};

pub mod configuration;
pub mod models;
pub mod schema;
pub mod statements;

pub fn init_configuration() {
    // Reset previous configuration set by the user.
    configuration::reset_configuration();
    // Set base memory size.
    configuration::set_memory_size(50);
    // TODO: set partition consecutive number to 1
    // TODO: Set compactions to 0
    // TODO: Set condensations to 0
}

pub fn init_processes() -> () {
    // WARN: THIS IS ONLY FOR TESTING, and should be removed when running.
    create_process(Process::new("P1".to_string(), 20, 10));
    create_process(Process::new("P2".to_string(), 6, 4));
    create_process(Process::new("P3".to_string(), 18, 9));
    create_process(Process::new("P4".to_string(), 4, 20));
    create_process(Process::new("P5".to_string(), 3, 10));
    create_process(Process::new("P6".to_string(), 12, 18));
    create_process(Process::new("P7".to_string(), 14, 17));
    create_process(Process::new("P8".to_string(), 8, 16));
    create_process(Process::new("P9".to_string(), 9, 1));
    create_process(Process::new("P10".to_string(), 10, 50));
}

pub fn clear_database() {
    // Remove everything BUT the processes, as this can be useful.
    delete_all_iteration_logs();
    delete_all_processes_logs();
    delete_all_processes_partitions();
    delete_all_storage_partitions();
    delete_all_storage_partitions_logs();
    delete_all_finished_processes();
    delete_all_condensations_logs();
    delete_all_compactions_logs();
}

fn delete_all_compactions_logs() -> () {
    use schema::compaction_log;

    let connection = establish_connection();
    diesel::delete(compaction_log::table)
        .execute(&connection)
        .expect("Could not delete compactions log");
}

fn delete_all_condensations_logs() -> () {
    use schema::condensation_log;

    let connection = establish_connection();
    diesel::delete(condensation_log::table)
        .execute(&connection)
        .expect("Could not delete condensation log");
}

fn delete_all_finished_processes() -> () {
    use schema::finished_process;

    let connection = establish_connection();
    diesel::delete(finished_process::table)
        .execute(&connection)
        .expect("Could not delete finished processes");
}

pub fn add_processes_to_memory() -> bool {
    // Select all ready processes
    println!("Selecting processes with ready state...");
    let processes = select_processes_with_state(StateEnum::Ready as i32);
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
    // Get all the current partitions and calculate the total memory they are
    // using
    let mut used_memory: i32 = 0;
    for partition in select_all_storage_partitions() {
        used_memory += partition.size;
    }

    // Get the remaining space if there is any
    let remaining_space = configuration::get_memory_size() - used_memory;

    if remaining_space > 0 {
        create_storage_partition(remaining_space);
    }
}

pub fn get_empty_storage_partition(process_size: i32) -> Option<models::StoragePartition> {
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
    let storage_partitions = select_all_storage_partitions_and_process_partitions();

    for i in 0..storage_partitions.len() {
        // Then the process might enter here.
        if storage_partitions[i].1.is_none() {
            // Then the process is the same size as the partition, it does not
            // remove or change the partition.
            if storage_partitions[i].0.size == process_size {
                return Some(select_storage_partition_with_id(storage_partitions[i].0.id));
            }
            // It means the partition is bigger than the process, so it has to
            // be removed and changed
            else if storage_partitions[i].0.size > process_size {
                // Delete the current partition
                delete_storage_partition_with_id(storage_partitions[i].0.id);
                // Create a new partition only for the process to fit in
                create_storage_partition_with_position(
                    storage_partitions[i].0.position,
                    process_size,
                );
                // Next to the created partition create a new one with the remaining space
                create_storage_partition_with_position(
                    storage_partitions[i].0.position + 1,
                    storage_partitions[i].0.size - process_size,
                );
                // Update all remaining partitions positions
                for e in i + 1..storage_partitions.len() {
                    // All partitons after are moved 1 space to the right
                    update_storage_partition_position(
                        storage_partitions[e].0.id,
                        storage_partitions[e].0.position + 1,
                        storage_partitions[e].0.position_end,
                        storage_partitions[e].0.position_end + storage_partitions[e].0.size,
                    );
                }
                // Return the new partiton for the process to fit in
                return Some(statements::select_storage_partition_with_position(
                    storage_partitions[i].0.position,
                ));
            }
        }
    }
    return None;
}

pub fn swap_process_partitions_with_empty_partitions() -> () {
    let mut partitions: Vec<(models::StoragePartition, Option<i32>)>;
    let mut made_compaction = false;
    // Get all the partitions
    partitions = statements::select_all_storage_partitions_and_process_partitions();
    // Get all the storage partitions and process partitions, ordered by
    // position and if it has a process or not.
    for i in 0..partitions.len() {
        // If the partition is empty, we can try to search a non-empty partition
        // and swap the positions of the partitions, so always the empty are
        // at the end.
        if partitions[i].1.is_none() {
            // From the current empty partition search the next non-empty
            // partition.
            for e in i..partitions.len() {
                // If it found a non-empty partition, do the swap
                if !partitions[e].1.is_none() {
                    println!("{} a {}", partitions[e].0.position, partitions[i].0.position);
                    println!("{}", partitions[e].0.position_start);
                    println!("{}", partitions[e].0.position_end);
                    println!("{}", partitions[i].0.position_start);
                    println!("{}", partitions[i].0.position_end);
                    update_storage_partition_position(
                        partitions[e].0.id,
                        partitions[i].0.position,
                        partitions[i].0.position_start,
                        partitions[i].0.position_start + partitions[e].0.size,
                    );
                    update_storage_partition_position(
                        partitions[i].0.id,
                        partitions[e].0.position,
                        partitions[i].0.position_start + partitions[e].0.size,
                        partitions[i].0.position_start
                            + partitions[e].0.size
                            + partitions[i].0.size,
                    );
                    // Save log of the part of the compaction done
                    statements::create_compaction_log(
                        partitions[e].0.number,
                        partitions[e].0.position_start,
                        partitions[i].0.position_start,
                    );
                    // After the swap re-select all storage partitions and
                    // process partitions, as their positions have changed
                    partitions = statements::select_all_storage_partitions_and_process_partitions();
                    made_compaction = true;
                    break;
                }
            }
        }
    }

    // If made at least 1 change to the storage positions, it means it did a
    // compaction, so update the number of compactions.
    if made_compaction {
        configuration::increment_compactions();
    }
}

pub fn merge_storage_partitions() {
    // Do al merges until there is no more merging done
    let mut has_finished_merging: bool;
    let mut storage_partitions: Vec<(models::StoragePartition, Option<i32>)>;
    loop {
        // Get all the partitions at the start, so the changes are reflected
        storage_partitions = statements::select_all_storage_partitions_and_process_partitions();
        // It assumes the merging has been done here, as if no change is done later
        has_finished_merging = true;
        for i in 0..storage_partitions.len() {
            // If there is no process here, start merging
            if storage_partitions[i].1.is_none() {
                let mut new_partition_size = 0;
                let mut partitions_changed: i32 = 0;

                // Start from the empty partition and see if the next partitions are empty
                for e in i..storage_partitions.len() {
                    if storage_partitions[e].1.is_none() {
                        let storage_partition =
                            select_storage_partition_with_id(storage_partitions[e].0.id);
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
                     0 1 2 3 4     0 1
                    +-+-+-+-+-+   +-+-+
                    |X|E|E|E|X+-->|X|X|
                    +-+-+-+-+-+   +-+-+
                     */
                    for e in i..i + partitions_changed as usize {
                        delete_storage_partition_with_id(storage_partitions[e].0.id);
                    }
                    // Create the new big partition from the first position, as if
                    // the partition was big enough from the start
                    create_storage_partition_with_position(i as i32, new_partition_size);
                    // Update all the remaining partition's positions
                    for e in i + partitions_changed as usize..storage_partitions.len() {
                        update_storage_partition_position(
                            storage_partitions[e].0.id,
                            e as i32 - partitions_changed + 1,
                            storage_partitions[e].0.position_start - new_partition_size,
                            (storage_partitions[e].0.position_start - new_partition_size)
                                + storage_partitions[e].0.size,
                        );
                    }
                    // Finally, update the condensation log for every partition that
                    // was changed and the final partition
                    let created_partition = select_storage_partition_with_position(i as i32);
                    for e in i..i + partitions_changed as usize {
                        create_condensation_log(
                            storage_partitions[e].0.number,
                            storage_partitions[e].0.size,
                            created_partition.number,
                            created_partition.size,
                        );
                    }
                    // And update the number of condensations made
                    configuration::increment_condensations();
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

    let partition_before = storage_partition::table
        .filter(storage_partition::position.eq(position - 1))
        .first::<models::StoragePartition>(&connection);

    if partition_before.is_ok() {
        diesel::insert_into(storage_partition::table)
            .values(models::NewStoragePartition {
                number: configuration::get_partition_consecutive_number(),
                position,
                position_start: partition_before.as_ref().unwrap().position_end,
                position_end: partition_before.as_ref().unwrap().position_end + size,
                size,
            })
            .execute(&connection)
            .expect("Could not create storage partition with position.");
    } else {
        diesel::insert_into(storage_partition::table)
            .values(models::NewStoragePartition {
                number: configuration::get_partition_consecutive_number(),
                position,
                position_start: 0,
                position_end: size,
                size,
            })
            .execute(&connection)
            .expect("Could not create storage partition with position.");
    }

    // Now that a new partition has been created, update the consecutive
    // number.
    configuration::increment_partition_consecutive_number();
}

pub fn create_storage_partition(size: i32) -> Option<models::StoragePartition> {
    use schema::storage_partition;

    if can_create_storage_partition(size) {
        let connection = establish_connection();

        let last_partition_position = storage_partition::table
            .order(storage_partition::position.desc())
            .first::<models::StoragePartition>(&connection);

        let new_storage_partition: models::NewStoragePartition;
        if last_partition_position.is_err() {
            new_storage_partition = models::NewStoragePartition {
                number: configuration::get_partition_consecutive_number(),
                position: 0,
                position_start: 0,
                position_end: size,
                size,
            };
        } else {
            new_storage_partition = models::NewStoragePartition {
                number: configuration::get_partition_consecutive_number(),
                position: last_partition_position.as_ref().unwrap().position + 1,
                position_start: last_partition_position.as_ref().unwrap().position_end,
                position_end: last_partition_position.as_ref().unwrap().position_end + size,
                size,
            };
        }
        // Now that a new partition has been created, update the consecutive
        // number.
        configuration::increment_partition_consecutive_number();

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

pub fn create_process(process: Process) -> () {
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
        .expect("Could not create process");
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

pub fn update_storage_partition_position(
    id: i32,
    position: i32,
    position_start: i32,
    position_end: i32,
) {
    use schema::storage_partition;

    let connection = establish_connection();
    println!("new position {}", position);
    diesel::update(storage_partition::table.find(id))
        .set((
            storage_partition::position.eq(position),
            storage_partition::position_start.eq(position_start),
            storage_partition::position_end.eq(position_end),
        ))
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

fn can_create_storage_partition(size: i32) -> bool {
    let partitions = select_all_storage_partitions();

    let mut used_memory: i32 = 0;
    for partition in partitions {
        used_memory += partition.size;
    }

    return used_memory + size <= configuration::get_memory_size();
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

pub fn create_finished_process(process_id: i32, partition_number: i32) -> () {
    use schema::finished_process;

    let connection = establish_connection();
    diesel::insert_into(finished_process::table)
        .values(&models::NewFinishedProcess {
            process_id,
            partition_number,
        })
        .execute(&connection)
        .expect("Could not insert finished process");
}

pub fn select_all_finished_processes() -> Vec<(models::FinishedProcess, models::Process)> {
    use schema::finished_process;
    use schema::process;

    let connection = establish_connection();
    finished_process::table
        .inner_join(process::table)
        .load::<(models::FinishedProcess, models::Process)>(&connection)
        .expect("Could not get all finished processes")
}

pub fn select_all_compaction_logs() -> Vec<models::CompactionLog> {
    use schema::compaction_log;

    let connection = establish_connection();
    compaction_log::table
        .load::<models::CompactionLog>(&connection)
        .expect("Could not get all compaction logs")
}

pub fn select_all_condensation_logs() -> Vec<models::CondensationLog> {
    use schema::condensation_log;

    let connection = establish_connection();
    condensation_log::table
        .load::<models::CondensationLog>(&connection)
        .expect("Could not get all compaction logs")
}

pub fn select_storage_partition_with_process_id(
    process_id: i32,
) -> (models::StoragePartition, models::ProcessPartition) {
    use schema::process_partition;
    use schema::storage_partition;

    let connection = establish_connection();
    storage_partition::table
        .inner_join(process_partition::table)
        .filter(process_partition::process_id.eq(process_id))
        .first::<(models::StoragePartition, models::ProcessPartition)>(&connection)
        .expect("Could not find storage partition with process id")
}
