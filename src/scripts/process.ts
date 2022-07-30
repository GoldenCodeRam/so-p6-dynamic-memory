import { createColumnHelper, type ColumnDef } from "@tanstack/svelte-table";

export enum State {
    PENDING,
    READY,
    RUNNING,
    FINISHED,
}

export function statusToString(state: State): string {
    switch (state) {
        case State.PENDING:
            return "Pendiente";
        case State.READY:
            return "Listo";
        case State.RUNNING:
            return "En ejecución";
        case State.FINISHED:
            return "Finalizado";
    }
}

export type Process = {
    name: string;
    time: number;
    size: number;
    state: State;
};

const columnHelper = createColumnHelper<Process>();

const defaultColumns: ColumnDef<Process>[] = [
    columnHelper.display({
        id: "actions",
        cell: (props) => props.row,
    }),
    columnHelper.accessor("name", {
        cell: (info) => info.getValue(),
        footer: (props) => props.column.id,
    }),
];
