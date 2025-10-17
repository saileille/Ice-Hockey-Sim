// Draw the competition screen of the competition given in the ID.
import { invoke } from "@tauri-apps/api/core";
import { initialiseAll, createCompSelect, goToChildCompetition } from "../initialise_generic";
import { createEventListener, createElement } from "../main";

// Draw any competition screen.
export const drawScreen = async (id: number) => {
    const json_s: string = await invoke("get_comp_screen_info", { id: id });
    const json = JSON.parse(json_s);

    initialiseAll();

    // If the competition is something like playoffs.
    // The JSON looks a bit different in this case.
    if (json.is_tournament_tree) {
        drawScreenTournament(json, id);
    }

    else if (json.format === null) {
        drawScreenParent(json, id);
    }
    else if (json.format.type === "RoundRobin") {
        drawScreenRoundRobin(json, id);
    }

    // Individual knockout rounds.
    else {
        drawScreenKnockoutRound(json, id);
    }
};

// Draw a screen for tournament-type competitions.
const drawScreenTournament = (json: any, id: number) => {
    createCompSelect(id);
    createEventListener("#child-comps", "change", goToChildCompetition);

    document.body.insertAdjacentHTML("beforeend", `
        <div id="tree"></div>
    `);

    const tree: HTMLDivElement = document.querySelector("#tree") as HTMLDivElement;
    for (const [index, round] of json.season.rounds.entries()) {
        const roundElement: HTMLDivElement = createElement("div", { "id": `round${index}` });
        createKnockoutPairElements(round.pairs, roundElement);
        tree.appendChild(roundElement);
    }

    drawSchedule(json, true);
};

// Draw a screen for parent competitions.
const drawScreenParent = (json: any, id: number) => {
    createCompSelect(id);
    drawRanking(json);
    createEventListener("#child-comps", "change", goToChildCompetition);
}

// Draw a screen for round robin competitions.
const drawScreenRoundRobin = (json: any, id: number) => {
    drawRoundRobinStandings(json);
    drawSchedule(json, false);
};

// Draw a screen for a knockout round.
const drawScreenKnockoutRound = (json: any, id: number) => {
    drawKnockoutPairs(json);
    drawSchedule(json, true);
};

// Draw the standings for round robin.
const drawRoundRobinStandings = (json: any) => {
    document.body.insertAdjacentHTML("beforeend", `
        <table id="standings"><tr>
            <th>Rank</th>
            <th>Name</th>
            <th>Games</th>
            <th>Wins</th>
            <th>OT Wins</th>
            <th>Draws</th>
            <th>OT Losses</th>
            <th>Losses</th>
            <th>Goals For</th>
            <th>Goals Against</th>
            <th>Goal Difference</th>
            <th>Points</th>
        </tr></table>
    `);

    const standings: HTMLTableElement = document.querySelector("#standings") as HTMLTableElement;
    for (const team of json.season.teams) {
        const row: HTMLTableRowElement = document.createElement("tr");

        row.appendChild(createElement("td", { "textContent": team.rank }));
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
    }
};

// Draw a competition schedule.
// Only previous and next matches for now.
const drawSchedule = (json: any, displaySeed: boolean) => {
    document.body.insertAdjacentHTML("beforeend", `
        <div id="previous-date"></div>
        <div id="previous"></div>
        <div id="next-date"></div>
        <div id="next"></div>
    `);

    let previousMatchDay: string = "";
    const previousGames: HTMLDivElement = document.querySelector("#previous") as HTMLDivElement;
    const previousDate: HTMLDivElement = document.querySelector("#previous-date") as HTMLDivElement;
    for (let i = json.season.played_games.length - 1; i >= 0; i--) {
        const match = json.season.played_games[i];
        if (previousMatchDay === "") { previousMatchDay = match.date; }
        else if (previousMatchDay !== match.date) { break; }

        let otString: string = "";
        if (match.had_overtime) {
            otString = " OT";
        }

        let matchString = `${match.home.name} ${match.home.goals} - ${match.away.goals}${otString} ${match.away.name}`;
        if (displaySeed) {
            matchString = `(${match.home.seed}.) ${matchString} (${match.away.seed}.)`;
        }

        previousGames.appendChild(createElement("div", { "textContent": matchString }));
    }

    if (previousMatchDay === "") {
        previousDate.textContent = "No previous games.";
    }
    else {
        previousDate.textContent = `Previous games from ${previousMatchDay}`;
    }

    let nextMatchDay: string = "";
    const nextGames: HTMLDivElement = document.querySelector("#next") as HTMLDivElement;
    const nextDate: HTMLDivElement = document.querySelector("#next-date") as HTMLDivElement;
    for (let i = json.season.upcoming_games.length - 1; i >= 0; i--) {
        const match = json.season.upcoming_games[i];
        if (nextMatchDay === "") { nextMatchDay = match.date; }
        else if (nextMatchDay !== match.date) { break; }

        let matchString = `${match.home.name} - ${match.away.name}`;
        if (displaySeed) {
            matchString = `(${match.home.seed}.) ${matchString} (${match.away.seed}.)`;
        }

        nextGames.appendChild(createElement("div", { "textContent": matchString }));
    }

    if (nextMatchDay === "") {
        nextDate.textContent = "No upcoming games.";
    }
    else {
        nextDate.textContent = `Next games on ${nextMatchDay}`;
    }
};

// Draw rankings for a competition.
const drawRanking = (json: any) => {
    document.body.insertAdjacentHTML("beforeend", `
        <table id="ranking"><tr>
            <th>Rank</th>
            <th>Team</th>
        </tr></table>
    `);

    const ranking: HTMLTableElement = document.querySelector("#ranking") as HTMLTableElement;
    for (const team of json.season.teams) {
        const row = document.createElement("tr");
        row.appendChild(createElement("td", { "textContent": team.rank }));
        row.appendChild(createElement("td", { "textContent": team.name }));
        ranking.appendChild(row);
    }
};

// Draw pairs of the knockout round.
const drawKnockoutPairs = (json: any) => {
    document.body.insertAdjacentHTML("beforeend", `
        <div id="pairs"></div>
    `);

    const pairs: HTMLDivElement = document.querySelector("#pairs") as HTMLDivElement;
    createKnockoutPairElements(json.season.knockout_round.pairs, pairs);
};

// Create elements of knockout pairs and append them to the given parent element.
const createKnockoutPairElements = (json_pairs: any, parentElement: HTMLDivElement) => {
    for (const pair of json_pairs) {
        parentElement.appendChild(createElement("div", {
            "textContent": `(${pair.home.seed}.) ${pair.home.name} ${pair.home.wins} - ${pair.away.wins} ${pair.away.name} (${pair.away.seed}.)`
        }));
    }
};