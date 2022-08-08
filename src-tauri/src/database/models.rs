use serde::Serialize;

use super::schema::{
    compaction_log, condensation_log, configuration, finished_process, process, process_log,
    process_partition, storage_partition, storage_partition_log,
};

#[derive(Insertable)]
#[table_name = "finished_process"]
pub struct NewFinishedProcess {
    pub process_id: i32,
    pub partition_number: i32,
}
#[derive(Queryable, Serialize)]
pub struct FinishedProcess {
    pub id: i32,
    pub process_id: i32,
    pub partition_number: i32,
}

#[derive(Insertable)]
#[table_name = "compaction_log"]
pub struct NewCompactionLog {
    pub iteration: i32,
    pub partition: i32,
    pub previous_position: i32,
    pub final_position: i32,
}
#[derive(Queryable, Serialize)]
pub struct CompactionLog {
    pub id: i32,
    pub iteration: i32,
    pub partition: i32,
    pub previous_position: i32,
    pub final_position: i32,
}

#[derive(Insertable)]
#[table_name = "condensation_log"]
pub struct NewCondensationLog {
    pub partition: i32,
    pub partition_size: i32,
    pub new_partition: i32,
    pub new_partition_size: i32,
}
#[derive(Queryable, Serialize)]
pub struct CondensationLog {
    pub id: i32,
    pub partition: i32,
    pub partition_size: i32,
    pub new_partition: i32,
    pub new_partition_size: i32,
}

#[derive(Queryable, Serialize)]
pub struct Process {
    pub id: i32,
    pub name: String,
    pub time: i32,
    pub size: i32,
    pub state: i32,
}

#[derive(Insertable)]
#[table_name = "process"]
pub struct NewProcess<'a> {
    pub name: &'a str,
    pub time: i32,
    pub size: i32,
    pub state: i32,
}

#[derive(Queryable, Serialize)]
pub struct StoragePartition {
    pub id: i32,
    pub number: i32,
    pub position: i32,
    pub size: i32,
}

#[derive(Insertable)]
#[table_name = "storage_partition"]
pub struct NewStoragePartition {
    pub number: i32,
    pub position: i32,
    pub size: i32,
}

#[derive(Queryable, Serialize)]
pub struct IterationLog {
    pub id: i32,
}

#[derive(Insertable)]
#[table_name = "storage_partition_log"]
pub struct NewStoragePartitionLog {
    pub iteration: i32,
    pub storage_partition_id: i32,
    pub position: i32,
    pub size: i32,
}

#[derive(Queryable, Serialize)]
pub struct StoragePartitionLog {
    pub id: i32,
    pub iteration: i32,
    pub storage_partition_id: i32,
    pub position: i32,
    pub size: i32,
}

#[derive(Insertable)]
#[table_name = "process_log"]
pub struct NewProcessLog {
    pub iteration: i32,
    pub process_id: i32,
    pub storage_partition_id: i32,
    pub time_remaining: i32,
    pub state: i32,
}

#[derive(Queryable, Serialize)]
pub struct ProcessLog {
    pub id: i32,
    pub iteration: i32,
    pub process_id: i32,
    pub storage_partition_id: i32,
    pub time_remaining: i32,
    pub state: i32,
}

#[derive(Queryable, Serialize, Insertable)]
#[table_name = "process_partition"]
pub struct ProcessPartition {
    pub process_id: i32,
    pub storage_partition_id: i32,
}

#[derive(Queryable, Serialize, Insertable)]
#[table_name = "configuration"]
pub struct Configuration {
    pub setting_id: i32,
    pub setting_value: String,
}
