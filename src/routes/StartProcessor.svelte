<script lang="ts">
    import { invoke } from "@tauri-apps/api/tauri";
    import type { CompactionLog } from "src/scripts/compactionLog";
    import type { CondensationLog } from "src/scripts/condensationLog";
    import type { Process } from "src/scripts/process";
    import LoadingSpinner from "../lib/LoadingSpinner.svelte";

    let compactions: number = 0;
    let condensations: number = 0;

    let finished_processes: Process[] = [];
    let compaction_logs: CompactionLog[] = [];
    let condensation_logs: CondensationLog[] = [];

    let ordered_condensation_logs: [string, any][] = [];
    let ordered_compaction_logs: [string, any][] = [];

    let hasFinished = false;
    let isLoading = false;

    async function startProcessor() {
        isLoading = true;
        await invoke("start_processor");

        for (const result of (await invoke("select_finished_processes")) as [
            any,
            Process
        ][]) {
            result[1].partition_number = result[0].partition_number;
            finished_processes.push(result[1]);
        }

        compactions = await invoke("select_compactions");
        condensations = await invoke("select_condensations");

        for (const result of (await invoke(
            "select_compaction_logs"
        )) as CompactionLog[]) {
            for (const process of finished_processes) {
                if (process.partition_number === result.partition) {
                    result.process = process;
                    break;
                }
            }
            console.log(result);

            compaction_logs.push(result);
        }

        for (const result of (await invoke(
            "select_condensation_logs"
        )) as CondensationLog[]) {
            condensation_logs.push(result);
        }
        ordered_condensation_logs = Object.entries(
            condensation_logs.reduce((accumulator, current_value) => {
                (accumulator[current_value.new_partition] =
                    accumulator[current_value.new_partition] || []).push(
                    current_value
                );
                return accumulator;
            }, {})
        );
        ordered_compaction_logs = Object.entries(
            compaction_logs.reduce((accumulator, current_value) => {
                (accumulator[current_value.iteration] =
                    accumulator[current_value.iteration] || []).push(
                    current_value
                );
                return accumulator;
            }, {})
        );

        hasFinished = true;
        isLoading = false;
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
        <div class="row w-100">
            <div class="col-3">
                <div class="p-2 m-2 bg-white rounded">
                    <div>
                        <h4>Condensaciones</h4>
                        <p class="text-center">{condensations}</p>
                    </div>
                    <div>
                        <h4>Compactaciones</h4>
                        <p class="text-center">{compactions}</p>
                    </div>
                </div>
            </div>
            <div class="col-3">
                <div class="p-2 m-2 bg-white rounded">
                    <h3>Procesos finalizados</h3>
                    <ul class="list-group list-group-numbered">
                        {#each finished_processes as finished_process}
                            <li class="list-group-item">
                                Nombre: {finished_process.name}
                                <br />
                                Tamaño: {finished_process.size}
                                <br />
                                Partición: {finished_process.partition_number}
                            </li>
                        {/each}
                    </ul>
                </div>
            </div>
            <div class="col-6">
                <div class="p-2 m-2 bg-white rounded">
                    <h3>Compactaciones</h3>
                    <ul class="list-group list-group-numbered">
                        {#each ordered_compaction_logs as ordered_compaction_log}
                            <li class="list-group-item bg-gray">
                                Compactación:
                                <div class="row p-2 m-2">
                                    <ul class="list-group">
                                        {#each ordered_compaction_log[1] as compaction}
                                            <li class="list-group-item">
                                                Partición: {compaction.partition}
                                                <br />
                                                Posición previa: {compaction.previous_position} a {compaction.previous_position + compaction.process.size}
                                                <br />
                                                Nueva posición: {compaction.final_position} a {compaction.final_position + compaction.process.size}
                                                <br />
                                            </li>
                                        {/each}
                                    </ul>
                                </div>
                            </li>
                        {/each}
                    </ul>
                </div>
            </div>
        </div>

        <div class="p-4 m-2 bg-white rounded w-100">
            <h3>Condensaciones</h3>
            <ul class="list-group list-group-numbered">
                {#each ordered_condensation_logs as ordered_condensation_log}
                    <li class="list-group-item bg-gray">
                        Condensación:
                        <div class="row p-2 m-2">
                            <div class="col-5">
                                <ul class="list-group">
                                    {#each ordered_condensation_log[1] as condensation}
                                        <li class="list-group-item">
                                            Partición: {condensation.partition}
                                            <br />
                                            Tamaño: {condensation.partition_size}
                                        </li>
                                    {/each}
                                </ul>
                            </div>
                            <div class="col-2 text-center">
                                <i class="bi bi-arrow-right fs-1" />
                            </div>
                            <div class="col-5">
                                <ul class="list-group">
                                    <li class="list-group-item">
                                        Partición: {ordered_condensation_log[0]}
                                        <br />
                                        Tamaño: {ordered_condensation_log[1][0]
                                            .new_partition_size}
                                    </li>
                                </ul>
                            </div>
                        </div>
                    </li>
                {/each}
            </ul>
        </div>
    {/if}
</div>
