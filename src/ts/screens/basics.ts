// Show the current date.
import { invoke } from "@tauri-apps/api/core";
import { createEventListener, createElement } from "../helpers.ts";
import { drawScreen as drawCompScreen } from "./competition.ts";
import { Listener } from "../types.ts";
import { onClickHomeScreen } from "./home.ts";
import { drawScreen as drawHomeScreen } from "./home.ts";

export const initialiseTopBar = () => {
    // Check if the basics have already been initialised.
    if (document.querySelector("#top-bar") !== null) {
        const comps = document.querySelector("#comps") as HTMLSelectElement;
        comps.value = "0";
        return;
    }

    document.body.innerHTML = `
        <div id="top-bar">
            <div id="date"></div>
            <button id="continue">Continue</button>
            <button id="home-screen">Home Screen</button>
        </div>
    `;

    displayDate();
    createCompSelect(document.querySelector("#top-bar") as HTMLDivElement, 0);

    createEventListener("#comps", "change", goToParentCompetition);
    createEventListener("#continue", "click", toNextDay);
    createEventListener("#home-screen", "click", onClickHomeScreen);
    initialiseContentScreen();
};

// Clear the content screen and return the element. Create one if it does not exist.
export const initialiseContentScreen = () => {
    let contentScreen = document.querySelector("#content-screen") as HTMLDivElement;
    if (contentScreen === null) {
        contentScreen = createElement("div", { id: "content-screen" });
        document.body.appendChild(contentScreen);
        return contentScreen;
    }

    contentScreen.innerHTML = "";
    return contentScreen;
};

const displayDate = async () => {
    const dateDiv: HTMLDivElement = document.querySelector("#date") as HTMLDivElement;
    const dateString: string = await invoke("get_date_string");
    dateDiv.textContent = dateString;
};

// Create competition selection dropdown and give items to it.
export const createCompSelect = async (element: HTMLDivElement, id: number) => {
    let query: string;
    if (id === 0) { query = "comps"; }
    else { query = "child-comps"; }

    element.insertAdjacentHTML("beforeend", `
        <select id="${query}"></select>
    `);

    const select: HTMLSelectElement = document.querySelector(`#${query}`) as HTMLSelectElement;

    let compData: Array<[string, string]>;
    if (id === 0) {
        compData = await invoke("get_comp_select_info");
    }
    else {
        compData = await invoke("get_child_comp_select_info", { id: id });
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

    dateDiv.textContent = dateString;
    drawHomeScreen();
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