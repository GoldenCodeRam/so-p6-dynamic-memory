<script lang="ts">
    import { invoke } from "@tauri-apps/api/tauri";
    import InfoModal from "../lib/InfoModal.svelte";
    import { Modal } from "bootstrap";

    let errorMessage = "";

    let processTimeInput: HTMLInputElement | null = null;
    let processNameInput: HTMLInputElement | null = null;
    let processSizeInput: HTMLInputElement | null = null;

    function createProcess() {
        if (processNameInput && processNameInput) {
            if (
                !isNaN(parseInt(processTimeInput.value)) &&
                processNameInput.value.length > 0
            ) {
                invoke("save_process", {
                    name: processNameInput.value,
                    time: parseInt(processTimeInput.value),
                    size: parseInt(processSizeInput.value),
                    isBlocked: true,
                }).then((response) => {
                    if (!response) {
                        errorMessage = `El nombre de proceso ${processNameInput.value} ya se encuentra en uso.`;
                        new Modal("#modal").show();
                    }
                    processNameInput.value = "";
                    processTimeInput.value = "";
                    processSizeInput.value = "";
                });
            }
        }
    }
</script>

<div
    class="d-flex flex-1-1-auto align-items-center justify-content-center bg-gray"
>
    <InfoModal id="modal" title="Ingresar otro nombre" content={errorMessage} />
    <div class="col col-md-6 m-5 p-5 text-center bg-white shadow rounded">
        <h2>Crear proceso</h2>
        <div class="text-start">
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
                    id="processSizeInput"
                    placeholder="tamaño"
                    bind:this={processSizeInput}
                />
                <label for="processSizeInput">Tamaño del proceso</label>
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
        </div>
        <button class="btn btn-primary" on:click={createProcess}
            >Crear proceso
        </button>
    </div>
</div>
