use crate::database::models;

use super::state;

pub struct Process {
    pub id: Option<i32>,
    pub name: String,
    pub time: i32,
    pub size: i32,

    pub state: Option<Box<dyn state::State>>,
}

impl Process {
    pub fn new(name: String, time: i32, size: i32) -> Process {
        Process {
            id: None,
            name,
            time,
            size,
            state: Some(Box::new(state::Pending {})),
        }
    }

    pub fn has_finished(&self) -> bool {
        self.state.as_ref().unwrap().has_finished()
    }

    pub fn process(&mut self) {
        println!(
            "Processing {} with status {}",
            self.name,
            self.state.as_ref().unwrap().get_state_number()
        );
        if let Some(s) = self.state.take() {
            self.state = Some(s.process(self))
        }
    }
}

pub fn create_process_from_model(process: &models::Process) -> Process {
    Process {
        id: Some(process.id),
        name: process.name.to_string(),
        time: process.time,
        size: process.size,
        state: state::get_state_from_enum(process.state),
    }
}
