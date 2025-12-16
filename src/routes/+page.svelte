<style>
    :global(a) {
        all: unset;
    }
</style>

<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";

    let name = $state("");
    let greetMsg = $state("");

    async function greet(event: Event) {
        event.preventDefault();
        // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
        greetMsg = await invoke("greet", { name });
    }
</script>

<main class="container">
    <button disabled>New Game</button>
    <button disabled>Load Game</button>
    <button><a href="/db">Database Editor</a></button>
</main>
