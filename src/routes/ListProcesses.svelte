<script lang="ts">
    import { invoke } from "@tauri-apps/api/tauri";
    import { Link } from "svelte-navigator";

    import LoadingSpinner from "../lib/LoadingSpinner.svelte";
    import { statusToString } from "../scripts/process";

    const processes = [];

    let hasFinishedLoading = false;
    let isTableNotEmpty = false;

    function updateProcessesTable() {
        hasFinishedLoading = false;
        invoke("select_all_processes").then((result: any) => {
            hasFinishedLoading = true;

            processes.length = 0;
            for (const process of result) {
                processes.push(process);
            }
            isTableNotEmpty = processes.length > 0;
        });
    }

    function deleteProcess(processId: number) {
        invoke("delete_process_with_id", {
            id: processId,
        }).then((result) => {
            if (result) {
                updateProcessesTable();
            }
        });
    }

    updateProcessesTable();
</script>

<div class="p-2 bg-gray d-flex flex-1-1-auto">
    {#if hasFinishedLoading}
        {#if isTableNotEmpty}
            <div class="container-fluid text-center p-4 bg-white rounded">
                <h3>Lista de procesos</h3>
                <table class="table table-hover">
                    <thead>
                        <tr>
                            <th scope="col">Id</th>
                            <th scope="col">Nombre</th>
                            <th scope="col">Tamaño</th>
                            <th scope="col">Tiempo</th>
                            <th scope="col">Estado</th>
                            <th scope="col">Acciones</th>
                        </tr>
                    </thead>
                    <tbody>
                        {#each processes as { id, name, time, size, state }}
                            <tr>
                                <th scope="row">{id}</th>
                                <td>{name}</td>
                                <td>{size}</td>
                                <td>{time}</td>
                                <td>{statusToString(state)}</td>
                                <td>
                                    <Link to="/editProcess/{id}">
                                        <button class="btn btn-sm btn-primary"
                                            >Editar</button
                                        >
                                    </Link>
                                    <button
                                        class="btn btn-sm btn-danger"
                                        on:click={() => deleteProcess(id)}
                                        >Eliminar</button
                                    >
                                </td>
                            </tr>
                        {/each}
                    </tbody>
                </table>
            </div>
        {:else}
            <div
                class="d-flex flex-1-1-auto justify-content-center align-items-center"
            >
                <div class="container text-center">
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        width="200"
                        height="200"
                        fill="var(--bs-gray)"
                        class="bi bi-inbox"
                        viewBox="0 0 16 16"
                    >
                        <path
                            d="M4.98 4a.5.5 0 0 0-.39.188L1.54 8H6a.5.5 0 0 1 .5.5 1.5 1.5 0 1 0 3 0A.5.5 0 0 1 10 8h4.46l-3.05-3.812A.5.5 0 0 0 11.02 4H4.98zm9.954 5H10.45a2.5 2.5 0 0 1-4.9 0H1.066l.32 2.562a.5.5 0 0 0 .497.438h12.234a.5.5 0 0 0 .496-.438L14.933 9zM3.809 3.563A1.5 1.5 0 0 1 4.981 3h6.038a1.5 1.5 0 0 1 1.172.563l3.7 4.625a.5.5 0 0 1 .105.374l-.39 3.124A1.5 1.5 0 0 1 14.117 13H1.883a1.5 1.5 0 0 1-1.489-1.314l-.39-3.124a.5.5 0 0 1 .106-.374l3.7-4.625z"
                        />
                    </svg>
                    <h3 class="text-secondary">Sin procesos</h3>
                </div>
            </div>
        {/if}
    {:else}
        <LoadingSpinner />
    {/if}
</div>
