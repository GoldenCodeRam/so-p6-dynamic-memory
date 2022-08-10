import type { Process } from "./process";

export type CompactionLog = {
    final_position: number;
    iteration: number;
    previous_position: number;
    partition: number;
    process: Process;
}
