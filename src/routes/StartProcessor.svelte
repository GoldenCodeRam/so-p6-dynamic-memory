<script lang="ts">
    import { invoke } from "@tauri-apps/api/tauri";
    import {
        State,
        statusToString,
        type PartitionLog,
        type ProcessLog,
    } from "../scripts/process";
    import LoadingSpinner from "../lib/LoadingSpinner.svelte";
    import {
        createSvelteTable,
        flexRender,
        getCoreRowModel,
        getSortedRowModel,
        type ColumnDef,
    } from "@tanstack/svelte-table";
    import type { TableOptions } from "@tanstack/svelte-table";
    import { writable } from "svelte/store";

    let hasFinished = false;
    let isLoading = false;

    let finalPartitions = null;

    let processLogs: ProcessLog[] = [];
    let partitionLogs: PartitionLog[] = [];

    let partitionLogsFilter: PartitionLog[] = [];

    async function startProcessor() {
        isLoading = true;
        await invoke("start_processor");
        finalPartitions = await invoke("select_all_storage_partitions");
        processLogs = (
            await invoke<ProcessLog[]>("select_all_process_logs")
        ).flatMap((process) => {
            return {
                name: process[0] as string,
                state: statusToString(process[1]),
                storagePartitionId:
                    process[2] == -1 ? "Sin partición" : process[2],
                timeRemaining: process[3],
            };
        });
        partitionLogs = await invoke<PartitionLog[]>(
            "select_all_storage_partition_logs"
        );

        isLoading = false;
        hasFinished = true;

        console.log(finalPartitions);
        console.log(processLogs);
        console.log(partitionLogs);

        rerenderer();
    }

    const processLogsColumns: ColumnDef<ProcessLog>[] = [
        {
            accessorKey: "name",
            cell: (info) => info.getValue(),
            header: () => "Nombre",
            footer: (info) => info.column.id,
        },
        {
            accessorKey: "state",
            cell: (info) => info.getValue(),
            header: () => "Estado",
            footer: (info) => info.column.id,
        },
        {
            accessorKey: "storagePartitionId",
            cell: (info) => info.getValue(),
            header: () => "Id de partición",
            footer: (info) => info.column.id,
        },
        {
            accessorKey: "timeRemaining",
            cell: (info) => info.getValue(),
            header: () => "Tiempo restante",
            footer: (info) => info.column.id,
        },
    ];

    const partitionLogsColumns: ColumnDef<PartitionLog>[] = [
        {
            accessorKey: "iteration",
            cell: (info) => info.getValue(),
            header: () => "Iteración",
            footer: (info) => info.column.id,
        },
        {
            accessorKey: "position",
            cell: (info) => info.getValue(),
            header: () => "Posición",
            footer: (info) => info.column.id,
        },
        {
            accessorKey: "size",
            cell: (info) => info.getValue(),
            header: () => "Tamaño",
            footer: (info) => info.column.id,
        },
    ];

    let processLogsSorting = [];
    let partitionLogsSorting = [];

    const setProcessLogsSorting = (updater) => {
        if (updater instanceof Function) {
            processLogsSorting = updater(processLogsSorting);
        } else {
            processLogsSorting = updater;
        }
        processLogsOptions.update((old) => ({
            ...old,
            state: {
                ...old.state,
                sorting: processLogsSorting,
            },
        }));
    };

    const setPartitionLogsSorting = (updater) => {
        if (updater instanceof Function) {
            partitionLogsSorting = updater(partitionLogsSorting);
        } else {
            partitionLogsSorting = updater;
        }
        partitionLogsOptions.update((old) => ({
            ...old,
            state: {
                ...old.state,
                sorting: partitionLogsSorting,
            },
        }));
    };

    const processLogsOptions = writable<TableOptions<ProcessLog>>({
        data: processLogs,
        columns: processLogsColumns,
        state: {
            sorting: processLogsSorting,
        },
        onSortingChange: setProcessLogsSorting,
        getCoreRowModel: getCoreRowModel(),
        getSortedRowModel: getSortedRowModel(),
        debugTable: true,
    });
    const partitionLogsOptions = writable<TableOptions<PartitionLog>>({
        data: partitionLogsFilter,
        columns: partitionLogsColumns,
        state: {
            sorting: partitionLogsSorting,
        },
        onSortingChange: setPartitionLogsSorting,
        getCoreRowModel: getCoreRowModel(),
        getSortedRowModel: getSortedRowModel(),
        debugTable: true,
    });

    const rerenderer = () => {
        processLogsOptions.update((options) => ({
            ...options,
            data: processLogs,
        }));
        partitionLogsOptions.update((options) => ({
            ...options,
            data: partitionLogsFilter,
        }));
    };

    const processLogsTable = createSvelteTable(processLogsOptions);
    const partitionLogsTable = createSvelteTable(partitionLogsOptions);

    let currentIterationNumber = 1;
    function nextIteraion() {
        currentIterationNumber++;
        partitionLogsFilter = partitionLogs.filter(
            (partition) => partition.iteration == currentIterationNumber
        );
        rerenderer();
    }

    function previousIteration() {
        currentIterationNumber--;
        partitionLogsFilter = partitionLogs.filter(
            (partition) => partition.iteration == currentIterationNumber
        );
        rerenderer();
    }
</script>

<div
    class="d-flex flex-1-1-auto bg-gray justify-content-center align-items-center flex-column p-3"
>
    <div class="bg-white px-5 py-4 rounded text-center shadow">
        <h2>Iniciar procesador</h2>
        <button class="btn btn-primary" on:click={startProcessor}
            >Iniciar</button
        >
    </div>

    {#if isLoading}
        <div class="mt-5">
            <LoadingSpinner />
        </div>
    {/if}

    {#if hasFinished}
        <div class="row mt-5 gap-4">
            <div class="col text-center p-4 bg-white rounded">
                <h3>Procesos</h3>
                <table class="table table-hover">
                    <thead>
                        {#each $processLogsTable.getHeaderGroups() as headerGroup}
                            <tr>
                                {#each headerGroup.headers as header}
                                    <th colspan={header.colSpan}>
                                        {#if !header.isPlaceholder}
                                            <div
                                                class:cursor-pointer={header.column.getCanSort()}
                                                class:select-none={header.column.getCanSort()}
                                                on:click={header.column.getToggleSortingHandler()}
                                            >
                                                <svelte:component
                                                    this={flexRender(
                                                        header.column.columnDef
                                                            .header,
                                                        header.getContext()
                                                    )}
                                                />
                                                {{
                                                    asc: "⬆️",
                                                    desc: "⬇️",
                                                }[
                                                    header.column
                                                        .getIsSorted()
                                                        .toString()
                                                ] ?? ""}
                                            </div>
                                        {/if}
                                    </th>
                                {/each}
                            </tr>
                        {/each}
                    </thead>
                    <tbody>
                        {#each $processLogsTable.getRowModel().rows as row}
                            <tr>
                                {#each row.getVisibleCells() as cell}
                                    <td>
                                        <svelte:component
                                            this={flexRender(
                                                cell.column.columnDef.cell,
                                                cell.getContext()
                                            )}
                                        />
                                    </td>
                                {/each}
                            </tr>
                        {/each}
                    </tbody>
                </table>
            </div>
            <div class="col text-center p-4 bg-white rounded">
                <h3>Particiones</h3>
                <div>
                    <button class="btn btn-primary" on:click={previousIteration}
                        >Atrás</button
                    >
                    <button class="btn btn-primary" on:click={nextIteraion}
                        >Siguiente</button
                    >
                </div>
                <table class="table table-hover">
                    <thead>
                        {#each $partitionLogsTable.getHeaderGroups() as headerGroup}
                            <tr>
                                {#each headerGroup.headers as header}
                                    <th colspan={header.colSpan}>
                                        {#if !header.isPlaceholder}
                                            <div
                                                class:cursor-pointer={header.column.getCanSort()}
                                                class:select-none={header.column.getCanSort()}
                                                on:click={header.column.getToggleSortingHandler()}
                                            >
                                                <svelte:component
                                                    this={flexRender(
                                                        header.column.columnDef
                                                            .header,
                                                        header.getContext()
                                                    )}
                                                />
                                                {{
                                                    asc: "⬆️",
                                                    desc: "⬇️",
                                                }[
                                                    header.column
                                                        .getIsSorted()
                                                        .toString()
                                                ] ?? ""}
                                            </div>
                                        {/if}
                                    </th>
                                {/each}
                            </tr>
                        {/each}
                    </thead>
                    <tbody>
                        {#each $partitionLogsTable.getRowModel().rows as row}
                            <tr>
                                {#each row.getVisibleCells() as cell}
                                    <td>
                                        <svelte:component
                                            this={flexRender(
                                                cell.column.columnDef.cell,
                                                cell.getContext()
                                            )}
                                        />
                                    </td>
                                {/each}
                            </tr>
                        {/each}
                    </tbody>
                </table>
            </div>
        </div>
    {/if}
</div>
