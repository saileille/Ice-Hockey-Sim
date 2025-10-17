// Show the current date.
import { invoke } from "@tauri-apps/api/core";
import { Listener, createEventListener, createElement } from "./main.ts";
import { drawScreen as drawCompScreen } from "./competitionScreen/main.ts";

export const initialiseAll = () => {
    document.body.innerHTML = `
        <div id="date"></div>
        <button id="continue">Continue</button>

    `;

    displayDate();
    createCompSelect(0);

    createEventListener("#comps", "change", goToParentCompetition);
    createEventListener("#continue", "click", toNextDay);
};

const displayDate = async () => {
    const dateDiv: HTMLDivElement = document.querySelector("#date") as HTMLDivElement;
    const dateString: string = await invoke("get_date_string");
    dateDiv.textContent = dateString;
};

// Create competition selection dropdown and give items to it.
export const createCompSelect = async (id: number) => {
    let query: string;
    if (id === 0) { query = "comps"; }
    else { query = "child-comps"; }

    document.body.insertAdjacentHTML("beforeend", `
        <select id="${query}"></select>
    `);

    const select: HTMLSelectElement = document.querySelector(`#${query}`) as HTMLSelectElement;

    let compData: Array<Array<string>>;
    if (id === 0) {
        compData = await invoke("get_all_full_competitions");
    }
    else {
        compData = await invoke("get_child_competitions", { id: id });
    }

    for (const comp of compData) {
        select.appendChild(createElement("option", {
            "value": comp[0],
            "textContent": comp[1]
        }));
    }
};

const toNextDay: Listener = async (_e: Event) => {
    const dateDiv: HTMLDivElement = document.querySelector("#date") as HTMLDivElement;
    const dateString: string = await invoke("go_to_next_day");

    // Should update a whole lot more.
    dateDiv.textContent = dateString;
};

const goToCompetition = (query: string) => {
    const compSelect: HTMLSelectElement = document.querySelector(query) as HTMLSelectElement;

    // This is the default option, we do not want that.
    if (compSelect.value === "0") { return; }
    drawCompScreen(Number(compSelect.value));
}

// Go to a competition page.
const goToParentCompetition: Listener = (_e: Event) => {
    goToCompetition("#comps");
};

export const goToChildCompetition: Listener = (_e: Event) => {
    goToCompetition("#child-comps");
};

initialiseAll();