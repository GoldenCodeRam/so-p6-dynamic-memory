use super::process::Process;

pub struct Memory {
    pub partitions: Vec<Partition>,
}

pub struct Partition {
    pub process: Box<Process>,
    pub size: i32,
}

impl Memory {
    fn new(processes: Vec<Process>) -> Memory {
        let mut memory = Memory {
            partitions: Vec::new(),
        };

        for process in processes {
            let boxed_process = Box::new(process);
            memory.partitions.push(Partition {
                size: boxed_process.size,
                process: boxed_process,
            });
        }
        return memory;
    }

    fn get_total_memory_size(self) -> i32 {
        let mut total_memory_size = 0;
        self.partitions
            .iter()
            .for_each(|process| total_memory_size += process.size);
        return total_memory_size;
    }
}
