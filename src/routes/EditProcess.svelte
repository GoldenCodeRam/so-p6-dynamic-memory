<script lang="ts">
    import { invoke } from "@tauri-apps/api/tauri";
    import { navigate } from "svelte-navigator";
    import { Modal } from "bootstrap";
    import InfoModal from "../lib/InfoModal.svelte";
    import type { Process } from "src/scripts/process";

    export let params: any;

    let processNameInput: HTMLInputElement | null = null;
    let processTimeInput: HTMLInputElement | null = null;
    let processSizeInput: HTMLInputElement | null = null;

    let errorMessage: String;

    invoke("select_process_with_id", { id: parseInt(params.id) }).then(
        (response) => {
            processNameInput.value = (response as Process).name;
            processTimeInput.value = (response as Process).time.toString();
            processSizeInput.value = (response as Process).size.toString();
        }
    );

    function editProcess() {
        if (processNameInput && processNameInput) {
            if (
                !isNaN(parseInt(processTimeInput.value)) &&
                processNameInput.value.length > 0
            ) {
                invoke("update_process_with_id", {
                    id: parseInt(params.id),
                    name: processNameInput.value,
                    time: parseInt(processTimeInput.value),
                    size: parseInt(processSizeInput.value),
                }).then((response) => {
                    if (response) {
                        navigate("/listProcesses");
                    } else {
                        errorMessage = `El nombre de proceso ${processNameInput.value} ya se encuentra en uso.`;
                        new Modal("#modal").show();
                    }
                });
            }
        }
    }
</script>

<div
    class="d-flex flex-1-1-auto align-items-center justify-content-center bg-gray"
>
    <InfoModal id="modal" title="Ingresar otro nombre" content={errorMessage} />
    <div class="container m-5 p-5 bg-white shadow rounded">
        <h2 class="text-center">Editar proceso</h2>
        <div class="input-group mb-3">
            <span class="input-group-text" id="processIdLabel">Id</span>
            <input
                type="text"
                disabled
                readonly
                class="form-control"
                placeholder="Id"
                aria-label="processIdLabel"
                aria-describedby="processIdLabel"
                value={params.id}
            />
        </div>

        <div class="form-floating mb-3">
            <input
                type="text"
                class="form-control"
                id="processNameInput"
                placeholder="nombre"
                bind:this={processNameInput}
            />
            <label for="processNameInput">Nombre del proceso</label>
        </div>
        <div class="form-floating mb-3">
            <input
                type="number"
                class="form-control"
                id="processTimeInput"
                placeholder="tiempo"
                bind:this={processTimeInput}
            />
            <label for="processTimeInput">Tiempo del proceso</label>
        </div>
        <div class="form-floating mb-3">
            <input
                type="number"
                class="form-control"
                id="processSizeInput"
                placeholder="tamaño"
                bind:this={processSizeInput}
            />
            <label for="processSizeInput">Tamaño del proceso</label>
        </div>
        <button class="btn btn-primary" on:click={editProcess}
            >Crear proceso
        </button>
    </div>
</div>
