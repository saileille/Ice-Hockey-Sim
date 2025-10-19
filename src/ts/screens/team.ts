// Team screen stuffs.
import { invoke } from "@tauri-apps/api/core";
import { initialiseBase } from "../initialise_base";
import { createElement, createEventListener, Listener } from "../helpers";

// Draw the screen of a given team.
export const drawScreen: Listener = async (e: Event) => {
    const target: HTMLSpanElement = e.target as HTMLSpanElement;
    const id = Number(target.id.replace("team", ""));

    const json_s: string = await invoke("get_team_screen_info", { id: id });
    const json = JSON.parse(json_s);

    initialiseBase();
    document.body.insertAdjacentHTML("beforeend", `
        <div>${json.name}</div>
    `);
    drawRoster(json.players);
};

// Draw the roster of a team.
const drawRoster = (players: any) => {
    document.body.insertAdjacentHTML("beforeend", `
        <select id="player-filters"></select>
        <table id="players"><tr>
            <th>Name</th>
            <th>Country</th>
            <th>Position</th>
            <th>Ability</th>
            <th>Seasons Left</th>
        </tr></table>
    `);

    const select = document.querySelector("#player-filters") as HTMLSelectElement;
    select.appendChild(createElement("option", { "value": "roster", "textContent": "Roster" }));
    select.appendChild(createElement("option", { "value": "approached", "textContent": "Approached" }));
    select.appendChild(createElement("option", { "value": "both", "textContent": "Roster + Approached" }));

    const roster = document.querySelector("#players") as HTMLTableElement;

    for (const player of players) {
        const row = document.createElement("tr");
        row.appendChild(createElement("td", { "textContent": player.name }));
        row.appendChild(createElement("td", { "textContent": player.country }));
        row.appendChild(createElement("td", { "textContent": player.position }));
        row.appendChild(createElement("td", { "textContent": player.ability }));
        row.appendChild(createElement("td", { "textContent": player.seasons_left }));

        roster.children[0].appendChild(row);
    }

    changePlayerFilter(roster, "roster");
    createEventListener("#player-filters", "change", onChangePlayerFilter);
};

// When the user changes the player filter of a team screen.
const onChangePlayerFilter: Listener = (e: Event) => {
    const roster = document.querySelector("#players") as HTMLTableElement;
    const select = e.target as HTMLSelectElement;
    const setting = select.value;

    changePlayerFilter(roster, setting);
};

const changePlayerFilter = (roster: HTMLTableElement, setting: string) => {
    for (const row of roster.children[0].children) {
        const r = row as HTMLTableRowElement;
        const seasonsLeftCell = r.children[4];

        // The header should always be displayed.
        if (seasonsLeftCell.tagName === "TH") { continue; }

        if (setting === "both") {
            r.style.display = "table-row";
            continue;
        }

        const seasonsLeft = seasonsLeftCell.textContent;
        if (setting === "roster") {
            if (seasonsLeft === "0") { r.style.display = "none"; }
            else { r.style.display = "table-row"; }
        }
        else if (setting === "approached") {
            if (seasonsLeft === "0") { r.style.display = "table-row"; }
            else { r.style.display = "none"; }
        }
        else { console.error(`Unknown filter setting ${setting}`); }
    }
};

// Draw a team field for any purpose.
export const createTeamField = (team_json: any, parentElement: HTMLElement) => {
    const team = createElement("span", {
        "textContent": team_json.name,
        "id": `team${team_json.id}`
    });

    // The event listener needs to be created later.
    // createEventListener(`#team${team_json.id}`, "click", goToTeamScreen)
    parentElement.appendChild(team);
};