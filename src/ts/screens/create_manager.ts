// The manager creation screen.
import { invoke } from "@tauri-apps/api/core";
import { initialiseContentScreen, createCompSelect } from "./basics";
import { createElement, createEventListener, linkListener } from "../helpers";
import { drawScreen as drawHomeScreen } from "./home";
import { Listener } from "../types";

// Draw the thing.
const drawScreen = async () => {
    const screen = initialiseContentScreen();
    screen.innerHTML = "<h1>Choose your competition and team</h1>";

    // This function must be awaited.
    await createCompSelect(screen, 0);

    // Removing the default option because we do not need it.
    const comps = document.querySelector("#comps") as HTMLSelectElement;
    if (comps.firstChild !== null) {
        comps.removeChild(comps.firstChild);
    }

    screen.insertAdjacentHTML("beforeend", `
        <select id="teams"></select>
        <button id="done">Done!</button>
    `);

    updateTeamSelection(Number(comps.value));
    createEventListener("#comps", "change", onChangeCompSelect);
    createEventListener("#done", "click", createManager);
};

const onChangeCompSelect: Listener = (e: Event) => {
    const compSelect = e.target as HTMLSelectElement;
    const compId = Number(compSelect.value);
    updateTeamSelection(compId);
};

const updateTeamSelection = async (id: Number) => {
    const teamSelect = document.querySelector("#teams") as HTMLSelectElement;
    const optionData: Array<[string, string]> = await invoke("get_team_select_info", { id: id });

    while (teamSelect.lastChild) { teamSelect.removeChild(teamSelect.lastChild); }

    for (const team of optionData) {
        teamSelect.appendChild(createElement("option", {
            "value": team[0],
            "textContent": team[1]
        }));
    }
};

const createManager: Listener = async (_e: Event) => {
    const teamSelect = document.querySelector("#teams") as HTMLSelectElement;
    await invoke("create_human_manager", { id: Number(teamSelect.value) });
    drawHomeScreen();
};

// This line of code needs to be in whatever TypeScript gets called first.
document.addEventListener("click", linkListener)

drawScreen();