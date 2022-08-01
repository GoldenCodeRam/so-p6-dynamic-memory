#![cfg_attr(
    all(not(debug_assertions), target_os = "window&s"),
    windows_subsystem = "windows"
)]

#[macro_use]
extern crate diesel;

use crate::database::models;

pub mod database;
mod model;

fn main() {
    // Start database base configuration
    database::init_configuration();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            save_process,
            delete_all_processes,
            select_all_processes,
            select_process_with_id,
            delete_process_with_id,
            update_process_with_id,
            start_processor,
            select_all_storage_partitions,
            select_all_process_logs,
            select_all_storage_partition_logs,
            change_memory_size,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

}

#[tauri::command]
fn save_process(name: String, time: i32, size: i32) -> bool {
    if database::check_process_name_is_unique(None, &name) {
        let process = model::process::Process::new(name, time, size);
        database::create_process(process).is_ok()
    } else {
        false
    }
}

#[tauri::command]
fn start_processor() -> bool {
    use model::process::create_process_from_model;

    database::clear_database();
    database::create_iteration_log().expect("Error creating iteration log");

    // This means there is no ready processes in the processor, so it has finished
    println!("Adding processes to memory...");
    if !database::add_processes_to_memory() {
        println!("Empty processes at start.");
        return true;
    }
    println!("Finished adding processes to memory.");
    // Here I generate a new partition with the remaining epty space
    database::create_storage_partition_from_remaining_space();
    // Log the start of the partitions
    database::create_storage_partition_logs();
    loop {
        database::create_iteration_log().expect("Error creating iteration log");
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

        // Try and add the remaining processes to the memory with the partitions
        // as they are.
        database::add_processes_to_memory();
        // After the processes are added or none could be added, merge the
        // remaining partitions
        database::merge_storage_partitions();
        // Now the partitions are merged try again to add processes, if it can
        // the processor continues, if it could'nt it means one of two things:
        // 1. The processor has no ready processes but it hasn't finished.
        // 2. The processor has no ready processes and it has finished
        if !database::add_processes_to_memory() {
            // The processor does not have ready processes and the partitions are
            // empty, the processor has finished
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
    database::statements::update_memory_size(size);
}

#[tauri::command]
fn processor_test() {
    // TESTING =================================================================
        delete_all_processes();
        save_process(String::from("P11"), 5, 11);
        save_process(String::from("P15"), 7, 15);
        save_process(String::from("P18"), 8, 18);
        save_process(String::from("P6"), 3, 6);
        save_process(String::from("P9"), 4, 9);
        save_process(String::from("P13"), 6, 13);
        save_process(String::from("P20"), 2, 20);
    // TESTING =================================================================
}