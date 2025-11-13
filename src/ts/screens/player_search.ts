// Player search screen, only free agents for now.

import { invoke } from "@tauri-apps/api/core";
import { initialiseContentScreen } from "./basics";
import { createImage, createElement, createLink, extractIdFromElement, resizeImages } from "../helpers";
import { Listener } from "../types/dom";
import { Player } from "../types/person";
import { HumanPackage } from "../types/team";

type PlayerFilter = "all" | "not-approached";

// Draw the player search screen.
export const drawScreen: Listener = async (_e: Event) => {
    const players: Array<Player> = await invoke("get_free_agents_package");
    const screen = initialiseContentScreen();

    const playerFilter = createElement("select", {}, [
        createElement("option", { "value": "all", "textContent": "All" }, []),
        createElement("option", { "value": "not-approached", "textContent": "Not Approached" }, []),
    ])

    const tbody = createElement("tbody", { "id": "players" }, []);

    for (const player of players) {
        tbody.appendChild(drawPlayer(player));
    }

    screen.append(
        playerFilter,
        createElement("table", {}, [
            createElement("thead", {}, [
                createElement("tr", {}, [
                    createElement("th", { "textContent": "Free Agents" }, []),
                    createElement("th", { "textContent": "Country" }, []),
                    createElement("th", { "textContent": "Position" }, []),
                    createElement("th", { "textContent": "Age" }, []),
                    createElement("th", { "textContent": "Ability" }, []),
                    createElement("th", { "textContent": "No. of Offers" }, []),
                ])
            ]),
            tbody
        ])
    );

    resizeImages();
    playerFilter.addEventListener("change", onChangePlayerFilter);
};

// Get a row of a single player.
const drawPlayer = (player: Player): HTMLTableRowElement => {
    const row = createElement("tr", {}, [
        createElement("td", {}, [createLink("span", "player", player.id, player.person.name)]),
        createElement("td", {}, [createImage(player.person.country, "block")]),
        createElement("td", { "textContent": player.position }, []),
        createElement("td", { "textContent": player.person.age }, []),
        createElement("td", { "textContent": player.ability }, []),
        createElement("td", { "textContent": player.person.offers.length }, []),
    ]);

    return row;
};

const onChangePlayerFilter: Listener = async (e: Event) => {
    const element = e.target as HTMLSelectElement;
    const tbody = document.querySelector("#players") as HTMLTableSectionElement;

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