export enum State {
    PENDING,
    READY,
    RUNNING,
    FINISHED
}

export function statusToString(state: State): string {
    switch(state) {
        case State.PENDING: return "Pendiente";
        case State.READY: return "Listo";
        case State.RUNNING: return "En ejecución";
        case State.FINISHED: return "Finalizado";
    }
}

export type Process = {
    name: string;
    time: number;
    size: number;
    is_blocked: boolean;
    state: State;
};
