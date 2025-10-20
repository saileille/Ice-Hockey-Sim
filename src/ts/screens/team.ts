// Team screen stuffs.
import { invoke } from "@tauri-apps/api/core";
import { initialiseContentScreen, initialiseTopBar } from "./basics";
import { createElement, createEventListener, createLink } from "../helpers";
import { Listener } from "../types";
import { drawScreen as drawPlayerScreen } from "./player";

type RosterSetting = "roster" | "approached" | "both";

type Player = {
    id: number,
    name: string,
    country: string,
    position: string,
    ability: number,
    seasons_left: number
};

type Team = {
    id: number,
    name: string,
    players: Array<Player>
};

// Draw the screen of a given team.
export const drawScreen = async (id: number) => {
    const json: string = await invoke("get_team_screen_info", { id: id });
    const team: Team = JSON.parse(json);

    initialiseTopBar();
    const screen = initialiseContentScreen();

    screen.insertAdjacentHTML("beforeend", `
        <div>${team.name}</div>
    `);
    drawRoster(screen, team.players);
};

// Draw the roster of a team.
const drawRoster = (screen: HTMLDivElement, players: Array<Player>) => {
    screen.insertAdjacentHTML("beforeend", `
        <select id="player-filters"></select>
        <table id="players"><tbody><tr>
            <th>Name</th>
            <th>Country</th>
            <th>Position</th>
            <th>Ability</th>
            <th>Seasons Left</th>
        </tr></tbody></table>
    `);

    const select = document.querySelector("#player-filters") as HTMLSelectElement;
    select.appendChild(createElement("option", { "value": "roster", "textContent": "Roster" }));
    select.appendChild(createElement("option", { "value": "approached", "textContent": "Approached" }));
    select.appendChild(createElement("option", { "value": "both", "textContent": "Roster + Approached" }));

    const roster = (document.querySelector("#players") as HTMLTableElement).children[0] as HTMLTableSectionElement;

    for (const player of players) {
        const row = document.createElement("tr");
        createLink("player", player.id, player.name, [document.createElement("td"), row]);
        row.appendChild(createElement("td", { "textContent": player.country }));
        row.appendChild(createElement("td", { "textContent": player.position }));
        row.appendChild(createElement("td", { "textContent": player.ability }));
        row.appendChild(createElement("td", { "textContent": player.seasons_left }));

        roster.appendChild(row);
        // createEventListener(`.player${player.id}`, "click", drawPlayerScreen);
    }

    changePlayerFilter(roster, "roster");
    createEventListener("#player-filters", "change", onChangePlayerFilter);
};

// When the user changes the player filter of a team screen.
const onChangePlayerFilter: Listener = (e: Event) => {
    const roster = (document.querySelector("#players") as HTMLTableElement).children[0] as HTMLTableSectionElement;
    const select = e.target as HTMLSelectElement;
    const setting = select.value as RosterSetting;

    changePlayerFilter(roster, setting);
};

const changePlayerFilter = (roster: HTMLTableSectionElement, setting: RosterSetting) => {
    for (const row of roster.children) {
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

        // Setting must be "approached".
        else {
            if (seasonsLeft === "0") { r.style.display = "table-row"; }
            else { r.style.display = "none"; }
        }
    }
};