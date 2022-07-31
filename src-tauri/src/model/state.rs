use super::process::Process;

#[derive(Copy, Clone)]
pub enum StateEnum {
    READY,
    RUNNING,
    FINISHED,
}

pub trait State {
    fn process(self: Box<Self>, process: &mut Process) -> Box<dyn State>;
    fn get_state_number(&self) -> i32;
    fn has_finished(&self) -> bool;
}

pub struct Ready;
impl State for Ready {
    fn process(self: Box<Ready>, _process: &mut Process) -> Box<dyn State> {
        Box::new(Running {})
    }

    fn get_state_number(&self) -> i32 {
        StateEnum::READY as i32
    }

    fn has_finished(&self) -> bool {
        false
    }
}

struct Running;
impl State for Running {
    fn process(self: Box<Running>, process: &mut Process) -> Box<dyn State> {
        process.time = std::cmp::max(0, process.time - 5);
        if process.time > 0 {
            Box::new(Ready {})
        } else {
            Box::new(Finished {})
        }
    }

    fn get_state_number(&self) -> i32 {
        StateEnum::RUNNING as i32
    }

    fn has_finished(&self) -> bool {
        false
    }
}

struct Finished;
impl State for Finished {
    fn process(self: Box<Self>, _process: &mut Process) -> Box<dyn State> {
        panic!("This should not happen.")
    }

    fn get_state_number(&self) -> i32 {
        StateEnum::FINISHED as i32
    }

    fn has_finished(&self) -> bool {
        true
    }
}

pub fn get_state_from_enum(value: i32) -> Option<Box<dyn State>> {
    match value {
        value if value == StateEnum::READY as i32 => Some(Box::new(Ready {})),
        value if value == StateEnum::RUNNING as i32 => Some(Box::new(Running {})),
        value if value == StateEnum::FINISHED as i32 => Some(Box::new(Finished {})),
        _ => panic!("State not recognized"),
    }
}
