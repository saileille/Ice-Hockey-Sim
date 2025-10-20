import { invoke } from "@tauri-apps/api/core";
import { Listener } from "../types";
import { initialiseTopBar, initialiseContentScreen } from "./basics";
import { createElement, createEventListener, createLink } from "../helpers";
import { drawScreen as drawTeamScreen } from "./team";

type ContractTeam = {
    name: string,
    id: number
};

type Contract = {
    start_date: string,
    end_date: string,
    seasons_left: number,
    team: ContractTeam
};

type Player = {
    name: string,
    country: string,
    position: string,
    ability: number,
    contract: Contract | null,
    offers: Array<Contract>
};

// Draw the screen of a given player.
export const drawScreen = async (id: number) => {
    const json: string = await invoke("get_player_screen_info", { id: id });
    const player: Player = await JSON.parse(json);

    initialiseTopBar();
    const screen = initialiseContentScreen();

    let pageTitle = `${player.name} (`;
    if (player.contract !== null) { pageTitle += `<span class="team${player.contract.team.id}">${player.contract.team.name}</span>, `; }
    pageTitle += `${player.position}, ${player.ability}, ${player.country})`;

    screen.insertAdjacentHTML("beforeend", `
        <div>${pageTitle}</div>
        <table id="contract"><tbody>
        </tbody></table>
    `);

    if (player.contract !== null) {
        drawContract(player.contract);
    }

    drawOffers(player.offers);
};

// Draw the current contract of the player.
const drawContract = (contract: Contract) => {
    const contractTable = (document.querySelector("#contract") as HTMLTableElement).children[0] as HTMLTableSectionElement;

    const firstRow = document.createElement("tr");
    firstRow.appendChild(createElement("th", { "textContent": "Current Contract" }));
    firstRow.appendChild(createElement("th", { "textContent": "Started" }));
    firstRow.appendChild(createElement("th", { "textContent": "Seasons Left" }));
    contractTable.appendChild(firstRow);

    const secondRow = document.createElement("tr");
    createLink("team", contract.team.id, contract.team.name, [document.createElement("td"), secondRow]);
    secondRow.appendChild(createElement("td", { "textContent": contract.start_date }));
    secondRow.appendChild(createElement("td", { "textContent": contract.end_date }));
    secondRow.appendChild(createElement("td", { "textContent": contract.seasons_left }));
    contractTable.appendChild(secondRow);
};

// Draw contract offers made for the player.
const drawOffers = (offers: Array<Contract>) => {
    console.log(JSON.stringify(offers));
    if (offers.length === 0) { return; }
    const contractTable = (document.querySelector("#contract") as HTMLTableElement).children[0] as HTMLTableSectionElement;

    const firstRow = document.createElement("tr");
    firstRow.appendChild(createElement("th", { "textContent": "Contract Offers" }));
    firstRow.appendChild(createElement("th", { "textContent": "Date", "colSpan": 2 }));
    firstRow.appendChild(createElement("th", { "textContent": "Seasons" }));
    contractTable.appendChild(firstRow);

    for (const offer of offers) {
        const row = document.createElement("tr");
        createLink("team", offer.team.id, offer.team.name, [document.createElement("td"), row]);
        row.appendChild(createElement("td", { "textContent": offer.start_date, "colSpan": 2 }));
        row.appendChild(createElement("td", { "textContent": offer.seasons_left }));
        contractTable.appendChild(row);
    }
};