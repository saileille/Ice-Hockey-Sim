// The manager creation screen.
import { invoke } from "@tauri-apps/api/core";
import { createTopLevelCompSelect, initialiseContentScreen, updateTopBar} from "./basics";
import { createElement, createEventListenerAsync, linkListener } from "../helpers";
import { drawScreen as drawHomeScreen } from "./home";
import { Listener } from "../types/dom";

// Draw the thing.
const drawScreen = async () => {
    const screen = initialiseContentScreen();

    screen.appendChild(
        createElement("h1", { "textContent": "Choose your competition and team" }, [])
    );

    const compSelect = await createTopLevelCompSelect(screen);

    // Removing the default option because we do not need it.
    if (compSelect.firstChild !== null) {
        compSelect.removeChild(compSelect.firstChild);
    }

    screen.append(
        createElement("select", { "id": "teams" }, []),
        createElement("button", { "id": "done", "textContent": "Done!" }, [])
    );

    await updateTeamSelection(Number(compSelect.value));
    compSelect.addEventListener("change", onChangeCompSelect);
    createEventListenerAsync("#done", "click", createManager);
};

const onChangeCompSelect: Listener = async (e: Event) => {
    const compSelect = e.target as HTMLSelectElement;
    const compId = Number(compSelect.value);
    await updateTeamSelection(compId);
};

const updateTeamSelection = async (id: Number) => {
    const teamSelect = document.querySelector("#teams") as HTMLSelectElement;
    const optionData: Array<[string, string]> = await invoke("team_select_package", { id: id });

    while (teamSelect.lastChild) { teamSelect.removeChild(teamSelect.lastChild); }

    const teams = [];
    for (const team of optionData) {
        teams.push(createElement("option", {
            "value": team[0],
            "textContent": team[1]
        }, []));
    }

    teamSelect.append(...teams);
};

const createManager: Listener = async (_e: Event) => {
    const teamSelect = document.querySelector("#teams") as HTMLSelectElement;
    const id = Number(teamSelect.value);

    await invoke("create_human_manager", { id: id });

    await updateTopBar();
    await drawHomeScreen();
};

// This line of code needs to be in whatever TypeScript gets called first.
document.addEventListener("click", linkListener)

await drawScreen();