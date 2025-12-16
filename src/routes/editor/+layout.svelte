<script lang="ts">
    let { children } = $props();
    import { invoke } from "@tauri-apps/api/core";

    async function newDatabase(event: Event) {
        event.preventDefault();
        await invoke("new_database");
    }

    async function loadDatabase(event: Event) {
        event.preventDefault();
        await invoke("load_database");
    }

    async function saveDatabase(event: Event) {
        event.preventDefault();
        await invoke("save_database");
    }

    async function toMainMenu(_: Event) {
        // event.preventDefault();
        await invoke("to_main_menu");
    }
</script>

<nav>
    <button onclick={newDatabase}>New Database</button>
    <button onclick={loadDatabase}>Load Database</button>
    <button onclick={saveDatabase}>Save Database</button>
    <button onclick={toMainMenu}><a href="/">Go Back</a></button>
</nav>
<ul>
    <li><a href="/editor/competition">Competitions</a></li>
    <li><a href="/editor/team">Teams</a></li>
</ul>

{@render children?.()}