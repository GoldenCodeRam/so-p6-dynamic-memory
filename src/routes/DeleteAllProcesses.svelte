<script>
    import { invoke } from "@tauri-apps/api/tauri";

    import { Modal } from "bootstrap";
    import InfoModal from "../lib/InfoModal.svelte";

    function deleteAllProcesses() {
        invoke("delete_all_processes").then((response) => {
            if (response) {
                new Modal("#modal").show();
            }
        });
    }
</script>

<div class="w-100 h-100 d-flex flex-column justify-content-center">
    <InfoModal
        id="modal"
        title="Realizado"
        content="Procesos eliminados correctamente."
    />
    <div class="container text-center p-5 bg-white rounded">
        <h3>¿Eliminar todos los procesos?</h3>
        <button
            class="btn btn-danger"
            type="button"
            on:click={deleteAllProcesses}>Eliminar</button
        >
    </div>
</div>
