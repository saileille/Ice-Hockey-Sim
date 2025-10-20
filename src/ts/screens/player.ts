import { invoke } from "@tauri-apps/api/core";
import { initialiseTopBar, initialiseContentScreen } from "./basics";
import { createElement, createEventListener, createLink } from "../helpers";
import { HumanInfo, HumanTeamInfo, Listener } from "../types";
import { drawScreen as drawHomeScreen } from "./home";

type ContractTeam = {
    name: string,
    id: number
};

export type Contract = {
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
    const player: Player = await invoke("get_player_screen_info", { id: id });
    const humanInfo: HumanInfo = await invoke("get_human_info");

    initialiseTopBar();
    const screen = initialiseContentScreen();

    let pageTitle = `${player.name} (`;
    if (player.contract !== null) { pageTitle += `<span class="team${player.contract.team.id}">${player.contract.team.name}</span>, `; }
    pageTitle += `${player.position}, ${player.ability}, ${player.country})`;

    screen.insertAdjacentHTML("beforeend", `
        <h1>${pageTitle}</h1>
        <table id="contract"><tbody>
        </tbody></table>
        <button id="offer-contract${id}">Offer Contract</button>
    `);

    if (player.contract !== null) {
        drawContract(player.contract);
    }

    drawOffers(player.offers);

    const offerContract = document.querySelector(`#offer-contract${id}`) as HTMLButtonElement;

    // Maybe some tooltip as to why the contract cannot be offered would be good...
    if (humanInfo.team === null || humanInfo.team.approached_players.includes(id) || humanInfo.team.actions_remaining <= 0) {
        offerContract.disabled = true;
    }
    else {
        createEventListener(`#offer-contract${id}`, "click", drawNegotiationScreen);
    }
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

// Draw the negotiation screen and get that player!
const drawNegotiationScreen: Listener = async (e: Event) => {
    const playerId = getPlayerIdFromContractOfferButton(e.target);
    if (playerId === 0) {
        drawHomeScreen();
        return;
    }

    initialiseTopBar();
    const screen = initialiseContentScreen();
    screen.innerHTML = `
        <label for="years">Seasons:</label>
        <select id="years">
            <option value="1">1</option>
            <option value="2">2</option>
            <option value="3">3</option>
            <option value="4">4</option>
        </select>
        <button id="offer-contract${playerId}">Offer</button>
    `;

    createEventListener(`#offer-contract${playerId}`, "click", offerContractToPlayer);
};

const offerContractToPlayer: Listener = async (e: Event) => {
    const playerId = getPlayerIdFromContractOfferButton(e.target);
    if (playerId === 0) {
        drawHomeScreen();
        return;
    }

    const humanInfo: HumanInfo = await invoke("get_human_info");
    const years = Number((document.querySelector("#years") as HTMLSelectElement).value);
    await invoke("offer_contract", { playerId: playerId, teamId: (humanInfo.team as HumanTeamInfo).id, years: years });
    drawScreen(playerId);
};

const getPlayerIdFromContractOfferButton = (target: EventTarget | null): number => {
    const elementId = (target as HTMLButtonElement).id;
    const regexMatch = elementId.match(/offer-contract([0-9]+)/);

    // This should never happen, but it calms TypeScript down.
    if (regexMatch === null) {
        return 0;
    }

    return Number(regexMatch[1]);
};