#![cfg_attr(
    all(not(debug_assertions), target_os = "window&s"),
    windows_subsystem = "windows"
)]

#[macro_use]
extern crate diesel;

use crate::database::models;

use self::database::configuration;

pub mod database;
mod model;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            save_process,
            delete_all_processes,
            select_all_processes,
            select_process_with_id,
            delete_process_with_id,
            update_process_with_id,
            start_processor,
            change_memory_size,
            select_finished_processes,
            select_compactions,
            select_condensations,
            select_compaction_logs,
            select_condensation_logs,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn save_process(name: String, time: i32, size: i32) -> bool {
    if database::check_process_name_is_unique(None, &name) {
        let process = model::process::Process::new(name, time, size);
        database::create_process(process);
        true
    } else {
        false
    }
}

#[tauri::command]
fn start_processor() -> bool {
    use model::process::create_process_from_model;

    // Start database base configuration
    database::init_configuration();

    database::clear_database();
    database::create_iteration_log();

    // This means there is no ready processes in the processor, so it has finished
    println!("Adding processes to memory...");
    if !database::add_processes_to_memory() {
        println!("Empty processes at start.");
        return true;
    }
    println!("Finished adding processes to memory.");
    // Here I generate a new partition with the remaining empty space, this
    // should only be run once, and it is when the processor starts.
    database::create_storage_partition_from_remaining_space();
    // Log the start of the partitions
    database::create_storage_partition_logs();
    loop {
        database::create_iteration_log();
        database::select_all_processes_from_processes_partitions()
            .iter()
            .map(|process| create_process_from_model(process))
            .for_each(|mut process| {
                process.process();
                database::update_process_with_id(process.id.unwrap(), &process)
                    .expect("Could not update process");
                database::create_process_log(process.id.unwrap());
            });
        database::create_storage_partition_logs();

        // Before we try to add a new process to the processor and its partitions,
        // we need to check if the partitions that the processor has can be
        // compacted, so start by swapping process partitions with empty partitions.
        println!("Swapping partitions...");
        database::swap_process_partitions_with_empty_partitions();
        println!("finished swapping partitions...");
        // After the swap, merge all the empty swapped partitions.
        println!("Merging partitions...");
        database::merge_storage_partitions();
        println!("finished Merging partitions...");
        // Try and add the remaining processes to the memory with the new big
        // compacted partition if it did that.
        //
        // If the processor could'nt add processes after the compactation it
        // means one of two things:
        // 1. The processor has no ready processes but it hasn't finished.
        // 2. The processor has no ready processes and it has finished.
        if !database::add_processes_to_memory() {
            // The processor does not have ready processes and the partitions are
            // empty, so the processor has finished.
            if database::select_all_processes_from_processes_partitions().len() == 0 {
                return true;
            }
        }
    }
}

#[tauri::command]
fn update_process_with_id(id: i32, name: String, time: i32, size: i32) -> bool {
    if database::check_process_name_is_unique(Some(id), &name) {
        let process = model::process::Process::new(name, time, size);
        database::update_process_with_id(id, &process).is_ok()
    } else {
        false
    }
}

#[tauri::command]
fn delete_all_processes() -> bool {
    database::delete_all_processes().is_ok()
}

#[tauri::command]
fn select_process_with_id(id: i32) -> Result<models::Process, bool> {
    let process = database::select_process_with_id(id);
    if process.is_ok() {
        Ok(process.unwrap())
    } else {
        Err(false)
    }
}

#[tauri::command]
fn select_all_processes() -> Result<Vec<models::Process>, bool> {
    let processes = database::select_all_processes();

    if processes.is_ok() {
        Ok(processes.unwrap())
    } else {
        Err(false)
    }
}

#[tauri::command]
fn select_all_storage_partitions() -> Vec<models::StoragePartition> {
    return database::select_all_storage_partitions();
}

#[tauri::command]
fn select_all_storage_partition_logs() -> Result<Vec<models::StoragePartitionLog>, bool> {
    let partitions = database::select_all_storage_partition_logs();

    if partitions.is_ok() {
        Ok(partitions.unwrap())
    } else {
        Err(false)
    }
}

#[tauri::command]
fn select_all_process_logs() -> Result<Vec<(String, i32, i32, i32)>, bool> {
    let process_logs = database::select_all_process_logs();

    if process_logs.is_ok() {
        Ok(process_logs.unwrap())
    } else {
        Err(false)
    }
}

#[tauri::command]
fn delete_process_with_id(id: i32) -> bool {
    database::delete_process_with_id(id).is_ok()
}

#[tauri::command]
fn change_memory_size(size: i32) {
    database::configuration::set_memory_size(size);
}

#[tauri::command]
fn select_finished_processes() -> Vec<(models::FinishedProcess, models::Process)> {
    database::select_all_finished_processes()
}

#[tauri::command]
fn select_compactions() -> i32 {
    configuration::get_compactions()
}

#[tauri::command]
fn select_condensations() -> i32 {
    configuration::get_condensations()
}

#[tauri::command]
fn select_compaction_logs() -> Vec<models::CompactionLog> {
    database::select_all_compaction_logs()
}

#[tauri::command]
fn select_condensation_logs() -> Vec<models::CondensationLog> {
    database::select_all_condensation_logs()
}
