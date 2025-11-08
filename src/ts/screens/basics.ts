// Show the current date.
import { invoke } from "@tauri-apps/api/core";
import { createElement } from "../helpers.ts";
import { drawScreen as drawCompScreen } from "./competition.ts";
import { HumanPackage, HumanTeamPackage, Listener } from "../types.ts";
import { onClickHomeScreen } from "./home.ts";
import { drawScreen as drawHomeScreen } from "./home.ts";
import { drawScreen as drawPlayerSearchScreen } from "./player_search.ts";

type TopBarPackage = {
    "date": string,
    "human": HumanPackage,
};

const initialiseTopBar = () => {
    // Check if the top bar has already been initialised.
    if (document.querySelector("#top-bar") !== null) { return; }

    const continueButton = createElement("button", { "textContent": "Continue" }, []);
    const homeScreenButton = createElement("button", { "textContent": "Home Screen" }, []);
    const scoutButton = createElement("button", { "textContent": "Scouting" }, []);

    const topBar = createElement("div", { "id": "top-bar" }, [
        createElement("div", { "id": "date" }, []),
        continueButton,
        homeScreenButton,
        scoutButton,
        createElement("span", {}, [
            "Actions remaining: ",
            createElement("span", { "id": "actions-remaining" }, []),
        ])
    ]);

    document.body.insertBefore(topBar, document.body.firstChild);
    createTopLevelCompSelect(topBar);

    continueButton.addEventListener("click", toNextDay);
    homeScreenButton.addEventListener("click", onClickHomeScreen);
    scoutButton.addEventListener("click", drawPlayerSearchScreen);
};

const resetCompSelect = (comps: HTMLSelectElement) => {
    comps.value = "0";
};

// Update the date and stuff in the top bar.
export const updateTopBar = async () => {
    initialiseTopBar();

    const topBarPackage: TopBarPackage = await invoke("get_top_bar_package");
    displayDate(topBarPackage.date);
    displayActionsRemaining(topBarPackage.human.team);

    // Making sure there is no obsolete information.
    initialiseContentScreen();
};

// Clear the content screen and return the element. Create one if it does not exist.
export const initialiseContentScreen = () => {
    let contentScreen = document.querySelector("#content-screen") as HTMLDivElement;
    if (contentScreen === null) {
        contentScreen = createElement("div", { id: "content-screen" }, []);
        document.body.appendChild(contentScreen);
        return contentScreen;
    }

    contentScreen.innerHTML = "";
    return contentScreen;
};

const displayDate = (date: string) => {
    const dateDiv: HTMLDivElement = document.querySelector("#date") as HTMLDivElement;
    dateDiv.textContent = date;
};

const displayActionsRemaining = (team: HumanTeamPackage | null) => {
    const actionsSpan = document.querySelector("#actions-remaining") as HTMLSpanElement;
    let actions = "0";
    if (team !== null) {
        actions = team.actions_remaining.toString();
    }

    actionsSpan.textContent = actions;
};

export const createTopLevelCompSelect = async (parent: Element): Promise<HTMLSelectElement> => {
    const comps: Array<[number, string]> = await invoke("get_comp_select_package");
    const compSelect = createCompSelect(comps, parent);
    // compSelect.id = "comps";
    return compSelect;
};

const createCompSelect = (comps: Array<[number, string]>, parent: Element): HTMLSelectElement => {
    const select = document.createElement("select");
    for (const comp of comps) {
        const option = createElement("option", {
            "value": comp[0],
            "textContent": comp[1]
        }, []);

        if (comp[0] === 0) {
            option.selected = true;
        }
        select.appendChild(option);
    }

    parent.appendChild(select);
    select.addEventListener("change", onCompSelectChange);

    return select;
};

export const createCompNav = (element: HTMLDivElement, compNav: Array<Array<[number, string]>>) => {
    for (const comps of compNav) {
        createCompSelect(comps, element);
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

    resetCompSelect(compSelect);
    drawCompScreen(id);
};