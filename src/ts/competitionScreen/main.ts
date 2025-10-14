// Draw the competition screen of the competition given in the ID.
import { invoke } from "@tauri-apps/api/core";
import { initialiseAll, populateCompSelect, goToChildCompetition } from "../initialiseGeneric";
import { createEventListener, createElement } from "../main";

// Draw any competition screen.
export const drawScreen = async (id: number) => {
    const json_s: string = await invoke("get_comp_screen_info", { id: id });
    const json = JSON.parse(json_s);

    initialiseAll();

    console.dir(json.format, {"showHidden": true});

    if (json.format === null) {
        drawScreenParent(json, id);
    }
    else if (json.format.type === "RoundRobin") {
        drawScreenRoundRobin(json, id);
    }

    // Knockouts...
    else {

    }
};

// Draw a competiton screen for parent competitions.
const drawScreenParent = (_json: any, id: number) => {
    document.body.insertAdjacentHTML("beforeend", `
        <select id="child-comps"></select>
    `);

    populateCompSelect(id);
    createEventListener("#child-comps", "change", goToChildCompetition);
}

// Draw a competition screen for round robin competitions.
const drawScreenRoundRobin = (json: any, _id: number) => {
    document.body.insertAdjacentHTML("beforeend", `
        <table id="standings"><tr>
            <td>Rank</td>
            <td>Name</td>
            <td>Games</td>
            <td>Wins</td>
            <td>OT Wins</td>
            <td>Draws</td>
            <td>OT Losses</td>
            <td>Losses</td>
            <td>Goals For</td>
            <td>Goals Against</td>
            <td>Goal Difference</td>
            <td>Points</td>
        </tr></table>
    `);

    const standings: HTMLTableElement = document.querySelector("#standings") as HTMLTableElement;
    json.season.teams.forEach((team: any, i: number) => {
        const row: HTMLTableRowElement = document.createElement("tr");

        row.appendChild(createElement("td", { "textContent": (i + 1).toString() }));
        row.appendChild(createElement("td", { "textContent": team.name }));
        row.appendChild(createElement("td", { "textContent": team.games.toString() }));
        row.appendChild(createElement("td", { "textContent": team.wins.toString() }));
        row.appendChild(createElement("td", { "textContent": team.ot_wins.toString() }));
        row.appendChild(createElement("td", { "textContent": team.draws.toString() }));
        row.appendChild(createElement("td", { "textContent": team.ot_losses.toString() }));
        row.appendChild(createElement("td", { "textContent": team.losses.toString() }));
        row.appendChild(createElement("td", { "textContent": team.goals_scored.toString() }));
        row.appendChild(createElement("td", { "textContent": team.goals_conceded.toString() }));
        row.appendChild(createElement("td", { "textContent": team.goal_difference.toString() }));
        row.appendChild(createElement("td", { "textContent": team.points.toString() }));

        standings.appendChild(row);
    });
};