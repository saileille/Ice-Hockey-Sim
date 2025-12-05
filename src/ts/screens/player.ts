import { invoke } from "@tauri-apps/api/core";
import { initialiseContentScreen, updateTopBar } from "./basics";
import { createElement, createEventListenerAsync, createTextImage, createLink } from "../helpers";
import { drawScreen as drawHomeScreen } from "./home";
import { Contract, Player } from "../types/person";
import { HumanPackage, HumanTeamPackage } from "../types/team";
import { Listener } from "../types/dom";

// Get the player screen title.
const getTitle = (player: Player): HTMLHeadingElement => {
    const element = document.createElement("h1");

    element.append(
        createTextImage(player.person.country),
        ` ${player.person.full_name} (`
    );

    if (player.person.contract !== null) {
        element.append(
            createLink("span", "team", player.person.contract.team.id, player.person.contract.team.name),
            ", "
        );
    }

    element.append(
        `${player.position}, ${player.ability})`
    );

    return element;
};

// Draw the screen of a given player.
export const drawScreen = async (id: number) => {
    const player: Player = await invoke("player_package", { id: id });
    const humanPackage: HumanPackage = await invoke("human_package");

    const screen = initialiseContentScreen();
    screen.append(
        getTitle(player),
        createElement("div", {"textContent": `Birthday: ${player.person.birthday}`}, []),
        drawContractTable(player),
    );

    // Contract offer can be made if...
    if (
        player.person.contract === null && // ...player does not have a contract,
        humanPackage.team !== null &&  // ...human is managing a team,
        !humanPackage.team.approached_players.includes(id) &&  // ...human's team has not approached the player,
        humanPackage.team.actions_remaining > 0    // ...and human team has actions remaining.
    ) {
        screen.appendChild(createElement("button", { "id": `offer-contract${id}`, "textContent": "Offer Contract" }, []));
        createEventListenerAsync(`#offer-contract${id}`, "click", drawNegotiationScreen);
    }
};

// Draw the contract table.
const drawContractTable = (player: Player) => {
    return createElement("table", {}, [
        createElement("tbody", {}, [
            ...drawContract(player.person.contract),
            ...drawOffers(player.person.offers),
        ])
    ])
}

// Draw the current contract of the player.
const drawContract = (contract: Contract | null): Array<HTMLTableRowElement> => {
    if (contract === null) {
        return [];
    }

    return [
        createElement("tr", {}, [
            createElement("th", { "textContent": "Current Contract" }, []),
            createElement("th", { "textContent": "Started" }, []),
            createElement("th", { "textContent": "Ends" }, []),
            createElement("th", { "textContent": "Season Left" }, []),
        ]),
        createElement("tr", {}, [
            createElement("td", {}, [createLink("span", "team", contract.team.id, contract.team.name)]),
            createElement("td", { "textContent": contract.start_date }, []),
            createElement("td", { "textContent": contract.end_date }, []),
            createElement("td", { "textContent": contract.seasons_left }, []),
        ])
    ];
};

// Draw contract offers made for the player.
const drawOffers = (offers: Array<Contract>): Array<HTMLTableRowElement> => {
    if (offers.length === 0) { return []; }

    const offerElements = [];
    offerElements.push(createElement("tr", {}, [
        createElement("th", { "textContent": "Contract Offers" }, []),
        createElement("th", { "textContent": "Date", "colSpan": 2 }, []),
        createElement("th", { "textContent": "Seasons" }, []),
    ]));

    for (const offer of offers) {
        offerElements.push(createElement("tr", {}, [
            createElement("td", {}, [createLink("span", "team", offer.team.id, offer.team.name)]),
            createElement("td", { "textContent": offer.start_date, "colSpan": 2 }, []),
            createElement("td", { "textContent": offer.seasons_left }, []),
        ]));
    }

    return offerElements;
};

// Draw the negotiation screen and get that player!
const drawNegotiationScreen: Listener = async (e: Event) => {
    const playerId = getPlayerIdFromContractOfferButton(e.target);
    if (playerId === 0) {
        await drawHomeScreen();
        return;
    }

    const screen = initialiseContentScreen();
    screen.append(
        ...drawYearSelection(),
        createElement("button", {
            "id": `offer-contract${playerId}`,
            "textContent": "Offer",
        }, [])
    );

    createEventListenerAsync(`#offer-contract${playerId}`, "click", offerContractToPlayer);
};

// Draw the year selection element.
const drawYearSelection = (): Array<HTMLElement> => {
    const elements = [
        createElement("label", {
            "for": "years",
            "textContent": "Seasons"
        }, []),
    ];

    const select = createElement("select", { "id": "years" }, []);
    for (let i = 1; i <= 4; i++) {
        const years = i.toString();
        select.appendChild(createElement("option", {
            "value": years,
            "textContent": years,
        }, []));
    }

    elements.push(select);
    return elements;
}

const offerContractToPlayer: Listener = async (e: Event) => {
    const playerId = getPlayerIdFromContractOfferButton(e.target);
    if (playerId === 0) {
        // Top bar should be updated here if actions remaining are displayed there...
        await drawHomeScreen();
        return;
    }
    const years = Number((document.querySelector("#years") as HTMLSelectElement).value);
    const humanPackage: HumanPackage = await invoke("human_package");
    await invoke("offer_contract", { playerId: playerId, teamId: (humanPackage.team as HumanTeamPackage).id, years: years });

    await updateTopBar(); // Needs to be updated as one action is used here.
    await drawScreen(playerId);
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