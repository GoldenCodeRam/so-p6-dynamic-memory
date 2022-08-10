use super::process::Process;

#[derive(Copy, Clone)]
pub enum StateEnum {
    Ready,
    ReadyInPartition,
    Running,
    Finished,
}

const PROCESSING_TIME: i32 = 1;

pub trait State {
    fn process(self: Box<Self>, process: &mut Process) -> Box<dyn State>;
    fn get_state_number(&self) -> i32;
}

pub struct Ready;
impl State for Ready {
    fn process(self: Box<Ready>, process: &mut Process) -> Box<dyn State> {
        process.time = std::cmp::max(0, process.time - PROCESSING_TIME);
        Box::new(Running {})
    }

    fn get_state_number(&self) -> i32 {
        StateEnum::Ready as i32
    }
}

struct ReadyInPartition;
impl State for ReadyInPartition {
    fn process(self: Box<Self>, process: &mut Process) -> Box<dyn State> {
        process.time = std::cmp::max(0, process.time - PROCESSING_TIME);
        Box::new(Running {})
    }

    fn get_state_number(&self) -> i32 {
        StateEnum::ReadyInPartition as i32
    }
}

struct Running;
impl State for Running {
    fn process(self: Box<Running>, process: &mut Process) -> Box<dyn State> {
        use crate::database;
        if process.time > 0 {
            Box::new(ReadyInPartition {})
        } else {
            // Add the finished process to the finished process list with the
            // information of the partition the process was in.
            database::create_finished_process(
                process.id.unwrap(),
                database::select_storage_partition_with_process_id(process.id.unwrap())
                    .0
                    .number,
            );
            // Remove the process from the processor, this is the process
            // partition.
            database::delete_process_partition_with_process_id(process.id.unwrap());
            Box::new(Finished {})
        }
    }

    fn get_state_number(&self) -> i32 {
        StateEnum::Running as i32
    }
}

struct Finished;
impl State for Finished {
    fn process(self: Box<Self>, _process: &mut Process) -> Box<dyn State> {
        Box::new(Self)
    }

    fn get_state_number(&self) -> i32 {
        StateEnum::Finished as i32
    }
}

pub fn get_state_from_enum(value: i32) -> Option<Box<dyn State>> {
    match value {
        value if value == StateEnum::Ready as i32 => Some(Box::new(Ready {})),
        value if value == StateEnum::ReadyInPartition as i32 => Some(Box::new(ReadyInPartition {})),
        value if value == StateEnum::Running as i32 => Some(Box::new(Running {})),
        value if value == StateEnum::Finished as i32 => Some(Box::new(Finished {})),
        _ => panic!("State not recognized"),
    }
}
