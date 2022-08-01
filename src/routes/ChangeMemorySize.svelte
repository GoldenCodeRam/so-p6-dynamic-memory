<script lang="ts">
    import { invoke } from "@tauri-apps/api/tauri";
    import InfoModal from "../lib/InfoModal.svelte";
    import { Modal } from "bootstrap";

    let errorMessage = "";

    let memorySizeInput: HTMLInputElement | null = null;

    function changeMemorySize() {
        if (memorySizeInput) {
            if (!isNaN(parseInt(memorySizeInput.value))) {
                invoke("change_memory_size", {
                    size: parseInt(memorySizeInput.value),
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
        <h2>Cambiar tamaño de la memoria</h2>
        <p>
            Se ha dejado un tamaño de memoria por defecto de 50, pero se puede
            cambiar aquí si es necesario.
        </p>
        <div class="text-start">
            <div class="form-floating mb-3">
                <input
                    type="number"
                    class="form-control"
                    id="memorySizeInput"
                    placeholder="Tamaño"
                    value="50"
                    bind:this={memorySizeInput}
                />
                <label for="processNameInput">Tamaño de memoria</label>
            </div>
        </div>
        <button class="btn btn-primary" on:click={changeMemorySize}>
            Cambiar tamaño
        </button>
    </div>
</div>
