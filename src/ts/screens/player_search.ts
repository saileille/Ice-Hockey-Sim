// Player search screen, only free agents for now.

import { invoke } from "@tauri-apps/api/core";
import { initialiseContentScreen } from "./basics";
import { Contract } from "./player";
import { createElement, createEventListener, createLink, extractIdFromElement } from "../helpers";
import { HumanPackage as HumanPackage, Listener } from "../types";

type Player = {
    id: number,
    name: string,
    country: string,
    position: string,
    ability: number,
    offers: Array<Contract>
};

type PlayerFilter = "all" | "not-approached";

// Draw the player search screen.
export const drawScreen: Listener = async (_e: Event) => {
    const players: Array<Player> = await invoke("get_free_agents_package");

    // initialiseTopBar();
    const screen = initialiseContentScreen();
    screen.innerHTML = `
        <select id="player-filter">
            <option value="all">All</option>
            <option value="not-approached">Not Approached</option>
        </select>
        <table id="players">
            <thead><tr>
                <th>Free Agents</th>
                <th>Country</th>
                <th>Position</th>
                <th>Ability</th>
                <th>No. of Offers</th>
            </tr></thead>
            <tbody></tbody>
        </table>
    `;

    const tbody = (document.querySelector("#players") as HTMLTableElement).children[1] as HTMLTableSectionElement;
    for (const player of players) {
        tbody.appendChild(drawPlayer(player));
    }

    createEventListener("#player-filter", "change", onChangePlayerFilter);

    // No need for this, since the all-inclusive option is default.
    // changePlayerFilter("all", tbody, humanInfo);
};

// Get a row of a single player.
const drawPlayer = (player: Player) => {
    const row = document.createElement("tr");
    createLink("span", "player", player.id, player.name, [document.createElement("td"), row]);
    row.appendChild(createElement("td", { "textContent": player.country }));
    row.appendChild(createElement("td", { "textContent": player.position }));
    row.appendChild(createElement("td", { "textContent": player.ability }));
    row.appendChild(createElement("td", { "textContent": player.offers.length }));
    return row;
};

const onChangePlayerFilter: Listener = async (e: Event) => {
    const element = e.target as HTMLSelectElement;
    const tbody = (document.querySelector("#players") as HTMLTableElement).children[1] as HTMLTableSectionElement;

    const humanPackage: HumanPackage = await invoke("get_human_package");
    await changePlayerFilter(element.value as PlayerFilter, tbody, humanPackage);
};

const changePlayerFilter = async (filter: PlayerFilter, tbody: HTMLTableSectionElement, humanPackage: HumanPackage) => {
    for (const row of tbody.children) {
        const r = row as HTMLTableRowElement;
        if (filter === "all" || humanPackage.team === null) {
            r.style.display = "table-row";
            continue;
        }

        const spanElement = r.children[0].children[0];

        const [_, id] = extractIdFromElement(spanElement) as [string, number];
        if (humanPackage.team.approached_players.includes(id)) {
            r.style.display = "none";
        }
        else {
            r.style.display = "table-row";
        }
    }
};