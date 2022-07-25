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
            select_all_storage_partition_logs
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

    // init processess partitions
    let processes = database::select_all_processes().expect("Error finding processes");

    database::delete_all_iteration_logs();
    database::delete_all_processes_logs();
    database::delete_all_processes_partitions();
    database::delete_all_storage_partitions();
    database::delete_all_storage_partitions_logs();

    database::create_iteration_log().expect("Error creating iteration log");
    for process in &processes {
        // Create iteration for the logs
        // Create all initial partitions for the processes to enter.
        let storage_partition = database::create_storage_partition(process.size)
            .expect("Something went wrong when creating the storage partition.");

        database::create_process_partition(models::ProcessPartition {
            process_id: process.id,
            storage_partition_id: storage_partition.id,
        });
    }

    while database::select_all_processes_from_processes_partitions()
        .expect("Error finding processes")
        .iter()
        .len()
        > 0
    {
        database::create_iteration_log().expect("Error creating iteration log");
        database::select_all_processes_from_processes_partitions()
            .expect("Error finding processes")
            .iter()
            .map(|process| create_process_from_model(process))
            .for_each(|mut process| {
                database::create_process_log(process.id.unwrap());
                if process.has_finished() {
                    database::delete_process_partition_with_process_id(process.id.unwrap());
                    database::merge_storage_partitions();
                } else {
                    process.process();
                    database::update_process_with_id(process.id.unwrap(), process)
                        .expect("Could not update process");
                }
            });
        database::create_storage_partition_logs();
    }

    //TODO delete all iteration log partitions
    //TODO delete all process log partitions
    return true;
}

#[tauri::command]
fn update_process_with_id(id: i32, name: String, time: i32, size: i32) -> bool {
    if database::check_process_name_is_unique(Some(id), &name) {
        let process = model::process::Process::new(name, time, size);
        database::update_process_with_id(id, process).is_ok()
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
fn select_all_storage_partitions() -> Result<Vec<models::StoragePartition>, bool> {
    let partitions = database::select_all_storage_partitions();

    if partitions.is_ok() {
        Ok(partitions.unwrap())
    } else {
        Err(false)
    }
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
fn select_all_process_logs() -> Result<Vec<(String, i32, i32, i32, i32)>, bool> {
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
