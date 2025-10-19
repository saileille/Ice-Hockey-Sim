// Team screen stuffs.
import { invoke } from "@tauri-apps/api/core";
import { initialiseBase } from "../initialise_base";
import { createElement, Listener } from "../helpers";

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
        <table id="roster"><tr>
            <th>Name</th>
            <th>Country</th>
            <th>Position</th>
            <th>Ability</th>
            <th>Seasons Left</th>
        </tr></table>
    `);

    const roster: HTMLTableElement = document.querySelector("#roster") as HTMLTableElement;

    for (const player of players) {
        const row = document.createElement("tr");

        row.appendChild(createElement("td", { "textContent": player.name }));
        row.appendChild(createElement("td", { "textContent": player.country }));
        row.appendChild(createElement("td", { "textContent": player.position }));
        row.appendChild(createElement("td", { "textContent": player.ability }));
        row.appendChild(createElement("td", { "textContent": player.seasons_left }));

        roster.appendChild(row);
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