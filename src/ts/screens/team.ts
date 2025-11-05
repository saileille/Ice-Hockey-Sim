// Team screen stuffs.
import { invoke } from "@tauri-apps/api/core";
import { initialiseContentScreen, initialiseTopBar } from "./basics";
import { createElement, createEventListener, createLink } from "../helpers";
import { Listener } from "../types";

type RosterSetting = "roster" | "approached" | "both";

type Player = {
    id: number,
    name: string,
    country: string,
    position: string,
    ability: number,
    seasons_left: number
};

type Manager = {
    name: string
};

type Team = {
    id: number,
    name: string,
    manager: Manager | null,
    players: Array<Player>
};

// Draw the screen of a given team.
export const drawScreen = async (id: number) => {
    const team: Team = await invoke("get_team_screen_info", { id: id });

    initialiseTopBar();
    const screen = initialiseContentScreen();

    screen.insertAdjacentHTML("beforeend", `
        <h1>${team.name}</h1>
        <div id="manager"></div>
    `);

    if (team.manager !== null) {
        (document.querySelector("#manager") as HTMLDivElement).textContent = `Manager: ${team.manager.name}`;
    }

    drawRoster(screen, team.players);
};

// Draw the roster of a team.
const drawRoster = (screen: HTMLDivElement, players: Array<Player>) => {
    screen.insertAdjacentHTML("beforeend", `
        <select id="player-filters">
            <option value="both">Roster + Approached</option>
            <option value="roster">Roster</option>
            <option value="approached">Approached</option>
        </select>
        <table id="players">
            <thead><tr>
                <th>Name</th>
                <th>Country</th>
                <th>Position</th>
                <th>Ability</th>
                <th>Seasons Left</th>
            </tr></thead>
            <tbody></tbody>
        </table>
    `);

    const roster = (document.querySelector("#players") as HTMLTableElement).children[1] as HTMLTableSectionElement;

    for (const player of players) {
        const row = document.createElement("tr");
        createLink("player", player.id, player.name, [document.createElement("td"), row]);
        row.appendChild(createElement("td", { "textContent": player.country }));
        row.appendChild(createElement("td", { "textContent": player.position }));
        row.appendChild(createElement("td", { "textContent": player.ability }));
        row.appendChild(createElement("td", { "textContent": player.seasons_left }));

        roster.appendChild(row);
    }

    createEventListener("#player-filters", "change", onChangePlayerFilter);
};

// When the user changes the player filter of a team screen.
const onChangePlayerFilter: Listener = (e: Event) => {
    const roster = (document.querySelector("#players") as HTMLTableElement).children[1] as HTMLTableSectionElement;
    const select = e.target as HTMLSelectElement;
    const setting = select.value as RosterSetting;

    changePlayerFilter(roster, setting);
};

const changePlayerFilter = (roster: HTMLTableSectionElement, setting: RosterSetting) => {
    for (const row of roster.children) {
        const r = row as HTMLTableRowElement;

        if (setting === "both") {
            r.style.display = "table-row";
            continue;
        }

        const seasonsLeftCell = r.children[4];
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