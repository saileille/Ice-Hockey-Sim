// The main screen of the game.
import { invoke } from "@tauri-apps/api/core";
import { initialiseContentScreen } from "./basics";
import { createElement } from "../helpers";
import { HumanPackage, RosterOverview } from "../types/team";
import { Listener } from "../types/dom";

export const drawScreen = async () => {
    const screen = initialiseContentScreen();
    const humanPackage: HumanPackage = await invoke("get_human_package");

    if (humanPackage.team !== null) {
        screen.appendChild(drawRosterOverview(humanPackage.team.roster_overview))
    }
};

// Draw a roster overview of the user's team.
const drawRosterOverview = (overview: RosterOverview): HTMLTableElement => {
    const data = {
        "GK": {
            "in_roster": 0,
            "approached": 0,
            "required": 2,
        },
        "LD": {
            "in_roster": 0,
            "approached": 0,
            "required": 4,
        },
        "RD": {
            "in_roster": 0,
            "approached": 0,
            "required": 4,
        },
        "LW": {
            "in_roster": 0,
            "approached": 0,
            "required": 4,
        },
        "C": {
            "in_roster": 0,
            "approached": 0,
            "required": 4,
        },
        "RW": {
            "in_roster": 0,
            "approached": 0,
            "required": 4,
        },
    };

    for (const player of overview) {
        const position = player.position;

        if (player.in_roster) data[position].in_roster++;
        else data[position].approached++;
    }

    const tbody = document.createElement("tbody");
    for (const [position, posData] of Object.entries(data)) {
        tbody.appendChild(createElement("tr", {}, [
            createElement("td", { "textContent": position }, []),
            createElement("td", { "textContent": posData.in_roster }, []),
            createElement("td", { "textContent": posData.approached }, []),
            createElement("td", { "textContent": posData.approached + posData.in_roster }, []),
            createElement("td", { "textContent": posData.required }, []),
        ]));
    }

    return createElement("table", {}, [
        createElement("thead", {}, [
            createElement("tr", {}, [
                createElement("th", {}, [
                    createElement("h3", {
                        "textContent": "Roster Overview",
                        "colSpan": 5
                    }, [])
                ])
            ]),
            createElement("tr", {}, [
                createElement("th", { "textContent": "Position" }, []),
                createElement("th", { "textContent": "In Roster" }, []),
                createElement("th", { "textContent": "Approached" }, []),
                createElement("th", { "textContent": "Total" }, []),
                createElement("th", { "textContent": "Required" }, []),
            ])
        ]),
        tbody
    ]);
};

// Listener of home screen button.
export const onClickHomeScreen: Listener = (_e: Event) => {
    drawScreen();
};