// Team screen stuffs.
import { invoke } from "@tauri-apps/api/core";
import { initialiseContentScreen } from "./basics";
import { createImage, createElement, createLink, resizeImages } from "../helpers";
import { RosterSetting, Team } from "../types/team";
import { Player } from "../types/person";
import { Listener } from "../types/dom";


// Draw the screen of a given team.
export const drawScreen = async (id: number) => {
    const team: Team = await invoke("get_team_screen_package", { id: id });

    const elements: Array<HTMLElement> = [
        createElement("h1", {"textContent": team.name}, [])
    ];

    if (team.manager !== null) {
        elements.push(createElement("div", { "textContent": `Manager: ${team.manager.person.name}` }, []));
    }

    const screen = initialiseContentScreen();
    screen.append(...elements);
    drawRoster(screen, team.players);
};

// Draw the roster of a team.
const drawRoster = (screen: HTMLDivElement, players: Array<Player>) => {
    const filterSelect = createElement("select", {}, [
        createElement("option", {"value": "both", "textContent": "Roster + Approached"}, []),
        createElement("option", {"value": "roster", "textContent": "Roster"}, []),
        createElement("option", {"value": "approached", "textContent": "Approached"}, []),
    ])

    const roster = createElement("tbody", { "id": "players" }, []);
    for (const player of players) {
        let seasonsLeft = 0;
        if (player.person.contract !== null)
            seasonsLeft = player.person.contract.seasons_left;

        roster.appendChild(createElement("tr", {}, [
            createElement("td", {}, [createLink("span", "player", player.id, player.person.name)]),
            createElement("td", {}, [createImage(player.person.country, "block")]),
            createElement("td", { "textContent": player.position }, []),
            createElement("td", { "textContent": player.person.age }, []),
            createElement("td", { "textContent": player.ability }, []),
            createElement("td", { "textContent": seasonsLeft }, []),
        ]));
    }

    screen.append(
        filterSelect,
        createElement("table", {}, [
            createElement("thead", {}, [
                createElement("tr", {}, [
                    createElement("th", { "textContent": "Name" }, []),
                    createElement("th", { "textContent": "Country" }, []),
                    createElement("th", { "textContent": "Position" }, []),
                    createElement("th", { "textContent": "Age" }, []),
                    createElement("th", { "textContent": "Ability" }, []),
                    createElement("th", { "textContent": "Seasons Left" }, []),
                ])
            ]),
            roster
        ])
    );

    resizeImages();
    filterSelect.addEventListener("change", onChangePlayerFilter);
};

// When the user changes the player filter of a team screen.
const onChangePlayerFilter: Listener = (e: Event) => {
    const roster = document.querySelector("#players") as HTMLTableSectionElement;
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

        const seasonsLeftCell = r.children[5];
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