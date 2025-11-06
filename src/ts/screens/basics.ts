// Show the current date.
import { invoke } from "@tauri-apps/api/core";
import { createEventListener, createElement, createLink } from "../helpers.ts";
import { drawScreen as drawCompScreen } from "./competition.ts";
import { Listener } from "../types.ts";
import { onClickHomeScreen } from "./home.ts";
import { drawScreen as drawHomeScreen } from "./home.ts";
import { drawScreen as drawPlayerSearchScreen } from "./player_search.ts";

type TopBarPackage = {
    "date": string,
};

const initialiseTopBar = () => {
    // Check if the basics have already been initialised.
    if (document.querySelector("#top-bar") !== null) {
        return;
    }

    document.body.innerHTML = `
        <div id="top-bar">
            <div id="date"></div>
            <button id="continue">Continue</button>
            <button id="home-screen">Home Screen</button>
            <button id="player-search">Scouting</button>
        </div>
    `;

    createTopLevelCompSelect(document.querySelector("#top-bar") as HTMLDivElement);
    createEventListener("#continue", "click", toNextDay);
    createEventListener("#home-screen", "click", onClickHomeScreen);
    createEventListener("#player-search", "click", drawPlayerSearchScreen);
};

const resetCompSelect = () => {
    const comps = document.querySelector("#comps") as HTMLSelectElement;
    comps.value = "0";
};

// Update the date and stuff in the top bar.
export const updateTopBar = async () => {
    initialiseTopBar();
    const topBarPackage: TopBarPackage = await invoke("get_top_bar_package");
    displayDate(topBarPackage.date);

    // Making sure there is no obsolete information.
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

const displayDate = async (date: string) => {
    const dateDiv: HTMLDivElement = document.querySelector("#date") as HTMLDivElement;
    // const dateString: string = await invoke("get_date_string");
    dateDiv.textContent = date;
};

export const createTopLevelCompSelect = async (element: Element) => {
    const comps = await invoke("get_comp_select_package") as Array<[string, string]>;

    const select = createCompSelect(comps, element);
    select.id = "comps";
};

const createCompSelect = (comps: Array<[string, string]>, parent: Element): HTMLSelectElement => {
    const select = document.createElement("select");
    for (const comp of comps) {
        select.appendChild(createElement("option", {
            "value": comp[0],
            "textContent": comp[1]
        }));
    }

    parent.appendChild(select);
    select.addEventListener("change", onCompSelectChange);

    return select;
};

export const createCompNav = (element: HTMLDivElement, compNav: Array<Array<[string, string]>>) => {
    // A button for the highest parent competition.
    createLink("button", "comp", Number(compNav[0][0][0]), compNav[0][0][1], [element]);

    // Dropdown menus for the rest.
    for (let i = 1; i < compNav.length; i++) {
        createCompSelect(compNav[i], element);
    }
};

const toNextDay: Listener = async (_e: Event) => {
    await invoke("go_to_next_day");
    updateTopBar();
    drawHomeScreen();
};

// Go to a given competition.
const onCompSelectChange: Listener = (e: Event) => {
    const compSelect = e.target as HTMLSelectElement;
    const id = Number(compSelect.value);

    // This is the default option, we do not want that.
    if (id === 0) { return; }

    resetCompSelect();
    drawCompScreen(id);
};

/* const goToCompetition = (query: string) => {
    const compSelect: HTMLSelectElement = document.querySelector(query) as HTMLSelectElement;

    // This is the default option, we do not want that.
    if (compSelect.value === "0") { return; }
    drawCompScreen(Number(compSelect.value));
} */