<script lang="ts">
    import { invoke } from "@tauri-apps/api/tauri";
import { statusToString } from "../scripts/process";
    import LoadingSpinner from "../lib/LoadingSpinner.svelte";

    let hasFinished = false;
    let isLoading = false;

    let finalPartitions = null;
    let processLogs = [];
    let partitionLogs = [];

    async function startProcessor() {
        isLoading = true;
        await invoke("start_processor");
        finalPartitions = await invoke("select_all_storage_partitions");
        processLogs = await invoke("select_all_process_logs");
        partitionLogs = await invoke("select_all_storage_partition_logs");

        isLoading = false;
        hasFinished = true;

        console.log(finalPartitions);
        console.log(processLogs);
        console.log(partitionLogs);
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
        <div
            class="bg-white shadow text-center container-fluid rounded px-5 py-4 mt-3"
        >
            <h3>Resultado</h3>
            <div>
                <h4>Partición final</h4>
                Tamaño: {finalPartitions[0].size}
            </div>
            <hr />
            <div style="max-height: 20em; overflow: auto">
                <h5>Procesos</h5>
                <table class="table table-hover">
                    <thead>
                        <tr>
                            <th scope="col">Iteración</th>
                            <th scope="col">Nombre</th>
                            <th scope="col">Estado</th>
                            <th scope="col">Partición</th>
                            <th scope="col">Tiempo restante</th>
                        </tr>
                    </thead>
                    <tbody>
                        {#each processLogs as process}
                            <tr>
                                <td>{process[2]}</td>
                                <td>{process[0]}</td>
                                <td>{statusToString(process[1])}</td>
                                <td>{process[3]}</td>
                                <td>{process[4]}</td>
                            </tr>
                        {/each}
                    </tbody>
                </table>
            </div>
            <div style="max-height: 20em; overflow: auto">
                <h5>Particiones</h5>
                <table class="table table-hover">
                    <thead>
                        <tr>
                            <th scope="col">Iteración</th>
                            <th scope="col">Tamaño</th>
                            <th scope="col">Partición</th>
                            <th scope="col">Posición</th>
                        </tr>
                    </thead>
                    <tbody>
                        {#each partitionLogs as log}
                            <tr>
                                <td>{log.iteration}</td>
                                <td>{log.size}</td>
                                <td>{log.storage_partition_id}</td>
                                <td>{log.position}</td>
                            </tr>
                        {/each}
                    </tbody>
                </table>
            </div>
        </div>
    {/if}
</div>
