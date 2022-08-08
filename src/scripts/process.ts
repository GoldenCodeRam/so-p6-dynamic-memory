
export enum State {
    READY,
    READY_IN_PARTITION,
    RUNNING,
    FINISHED,
}

export function statusToString(state: State): string {
    switch (state) {
        case State.READY:
            return "Listo";
        case State.READY_IN_PARTITION:
            return "Listo en partición";
        case State.RUNNING:
            return "En ejecución";
        case State.FINISHED:
            return "Finalizado";
    }
}

export type ProcessLog = {
    name: string;
    state: string;
    storagePartitionId: number;
    timeRemaining: number;
};

export type PartitionLog = {
    iteration: number;
    position: number;
    size: number;
    storagePartitionId: number;
};

export type Process = {
    name: string;
    time: number;
    size: number;
    state: State;
    partition_number: number;
};
