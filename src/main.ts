import { invoke } from "@tauri-apps/api/core";

let greetInputEl: HTMLInputElement | null;
let greetMsgEl: HTMLElement | null;

/*const greet = async () => {
    if (greetMsgEl && greetInputEl) {
        // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
        greetMsgEl.textContent = await invoke("greet", {
            name: greetInputEl.value,
        });
    }
};*/

const testGame = async () => {
    if (greetMsgEl && greetInputEl) {
        let texts: Array<string> = await invoke("test_game");
        greetMsgEl.textContent = texts[0];
        console.log(texts[1]);
    }
};

const testComp = async () => {
    let texts: String = await invoke("test_comp");
    console.log(texts);
};

window.addEventListener("DOMContentLoaded", () => {
    greetInputEl = document.querySelector("#greet-input");
    greetMsgEl = document.querySelector("#greet-msg");
    document.querySelector("#greet-form")?.addEventListener("submit", (e) => {
        e.preventDefault();
        // testGame();
        testComp();
    });
});