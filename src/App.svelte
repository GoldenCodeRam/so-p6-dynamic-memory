<script lang="ts">
    import { Link, Router, Route } from "svelte-navigator";

    import CreateProcess from "./routes/CreateProcess.svelte";
    import MainMenu from "./routes/MainMenu.svelte";
    import DeleteAllProcesses from "./routes/DeleteAllProcesses.svelte";
    import ListProcesses from "./routes/ListProcesses.svelte";
    import EditProcess from "./routes/EditProcess.svelte";
    import StartProcessor from "./routes/StartProcessor.svelte";
</script>

<Router>
    <div
        class="offcanvas offcanvas-start"
        data-bs-scroll="true"
        tabindex="-1"
        id="offcanvas"
        aria-labelledby="offcanvasLabel"
    >
        <div class="offcanvas-header border-bottom">
            <h5 class="offcanvas-title" id="offcanvasLabel">Menú principal</h5>
            <button
                type="button"
                class="btn-close"
                data-bs-dismiss="offcanvas"
                aria-label="Close"
            />
        </div>
        <div class="offcanvas-body">
            <ul class="nav nav-pills flex-column mb-auto gap-1">
                <li class="nav-item" data-bs-dismiss="offcanvas">
                    <Link to="createProcess" class="nav-link border rounded-3">
                        Crear proceso
                    </Link>
                </li>
                <li class="nav-item" data-bs-dismiss="offcanvas">
                    <Link to="listProcesses" class="nav-link border rounded-3">
                        Listar procesos
                    </Link>
                </li>
                <li class="nav-item" data-bs-dismiss="offcanvas">
                    <Link to="startProcessor" class="nav-link border rounded-3">
                        Iniciar procesador
                    </Link>
                </li>
                <hr />
                <h4 class="m-0 p-0">Configuración</h4>
                <li class="nav-item" data-bs-dismiss="offcanvas">
                    <Link
                        to="deleteAllProcesses"
                        class="nav-link border rounded-3"
                    >
                        Eliminar todos los procesos
                    </Link>
                </li>
            </ul>
        </div>
    </div>

    <main class="w-100 h-100 d-flex flex-column">
        <div class="container-fluid bg-white shadow-sm z-index-1">
            <header
                class="d-flex justify-content-between align-items-center py-3 mx-5"
            >
                <button
                    class="btn btn-primary"
                    type="button"
                    data-bs-toggle="offcanvas"
                    data-bs-target="#offcanvas"
                    aria-controls="offcanvas"
                >
                    Menú principal
                </button>
                <h3 class="m-0 p-0">Procesador</h3>
            </header>
        </div>
        <Route path="/">
            <MainMenu />
        </Route>
        <Route path="createProcess">
            <CreateProcess />
        </Route>
        <Route path="/editProcess/:id" let:params>
            <EditProcess {params} />
        </Route>
        <Route path="listProcesses">
            <ListProcesses />
        </Route>
        <Route path="/startProcessor">
            <StartProcessor />
        </Route>
        <Route path="deleteAllProcesses">
            <DeleteAllProcesses />
        </Route>
    </main>
</Router>
