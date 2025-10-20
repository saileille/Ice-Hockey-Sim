// Draw the competition screen of the competition given in the ID.
import { invoke } from "@tauri-apps/api/core";
import { initialiseTopBar, createCompSelect, goToChildCompetition, initialiseContentScreen } from "./basics";
import { createEventListener, createElement, createLink } from "../helpers";
import { drawScreen as drawTeamScreen } from "./team";

type Format = {
    round_robin: RoundRobinFormat | null,
    knockout_round: KnockoutRoundFormat | null,
    match_rules: MatchRules,
    type: string
};

type RoundRobinFormat = {
    rounds: number,
    extra_matches: number,
    points_for_win: number,
    points_for_ot_win: number,
    points_for_draw: number,
    points_for_ot_loss: number,
    points_for_loss: number,
};

type KnockoutRoundFormat = {
    wins_required: number
};

type MatchRules = {
    periods: number,
    period_length: number,
    overtime_length: number,
    continuous_overtime: boolean
};

type Team = {
    id: number,
    name: string,
    rank: string,
    games: number,
    wins: number,
    ot_wins: number,
    draws: number,
    ot_losses: number,
    losses: number,
    total_wins: number,
    total_losses: number,
    goals_scored: number,
    goals_conceded: number,
    goal_difference: number,
    points: number,
    seed: number
};

type KnockoutRound = {
    pairs: Array<KnockoutPair>
};

type KnockoutPair = {
    home: KnockoutTeam,
    away: KnockoutTeam
};

type KnockoutTeam = {
    id: number,
    name: string,
    wins: number,
    seed: number
};

type Game = {
    home: GameTeam,
    away: GameTeam,
    date: string,
    had_overtime: boolean,
    is_over: boolean
};

type GameTeam = {
    id: number,
    name: string,
    seed: number,
    goals: number
};

type Season = {
    name: string,
    teams: Array<Team>,
    knockout_round: KnockoutRound | null,
    rounds: Array<KnockoutRound> | undefined,
    upcoming_games: Array<Game>,
    played_games: Array<Game>
};

type Competition = {
    name: string,
    full_name: string,
    format: Format | null,
    season: Season,
    child_comp_ids: Array<number>,
    parent_comp_id: number,
    is_tournament_tree: boolean
};

// Draw any competition screen.
export const drawScreen = async (id: number) => {
    const json: string = await invoke("get_comp_screen_info", { id: id });
    const comp: Competition = JSON.parse(json);

    initialiseTopBar();
    const screen = initialiseContentScreen();

    // If the competition is something like playoffs.
    // The JSON looks a bit different in this case.
    if (comp.season.rounds !== undefined) {
        drawScreenTournament(screen, comp, comp.season.rounds, id);
    }

    else if (comp.format === null) {
        drawScreenParent(screen, comp, id);
    }
    else if (comp.format.type === "RoundRobin") {
        drawScreenRoundRobin(screen, comp);
    }

    // Individual knockout rounds.
    else {
        drawScreenKnockoutRound(screen, comp);
    }
};

// Draw a screen for tournament-type competitions.
const drawScreenTournament = (screen: HTMLDivElement, comp: Competition, rounds: Array<KnockoutRound>, id: number) => {
    createCompSelect(screen, id);
    createEventListener("#child-comps", "change", goToChildCompetition);

    screen.insertAdjacentHTML("beforeend", `
        <div id="tree"></div>
    `);

    const tree: HTMLDivElement = document.querySelector("#tree") as HTMLDivElement;
    for (const [index, round] of rounds.entries()) {
        const roundElement: HTMLDivElement = createElement("div", { "id": `round${index}` });
        createKnockoutPairElements(round.pairs, roundElement);
        tree.appendChild(roundElement);
    }

    drawSchedule(screen, comp.season, true);
};

// Draw a screen for parent competitions.
const drawScreenParent = (screen: HTMLDivElement, comp: Competition, id: number) => {
    createCompSelect(screen, id);
    drawRanking(screen, comp.season.teams);
    createEventListener("#child-comps", "change", goToChildCompetition);
}

// Draw a screen for round robin competitions.
const drawScreenRoundRobin = (screen: HTMLDivElement, comp: Competition) => {
    createCompSelect(screen, comp.parent_comp_id);
    createEventListener("#child-comps", "change", goToChildCompetition);

    drawRoundRobinStandings(screen, comp.season.teams);
    drawSchedule(screen, comp.season, false);
};

// Draw a screen for a knockout round.
const drawScreenKnockoutRound = (screen: HTMLDivElement, comp: Competition) => {
    createCompSelect(screen, comp.parent_comp_id);
    createEventListener("#child-comps", "change", goToChildCompetition);

    drawKnockoutPairs(screen, (comp.season.knockout_round as KnockoutRound).pairs);
    drawSchedule(screen, comp.season, true);
};

// Draw the standings for round robin.
const drawRoundRobinStandings = (screen: HTMLDivElement, teams: Array<Team>) => {
    screen.insertAdjacentHTML("beforeend", `
        <table id="standings"><tbody><tr>
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
        </tr></tbody></table>
    `);

    const standings = (document.querySelector("#standings") as HTMLTableElement).children[0] as HTMLTableSectionElement;
    for (const team of teams) {
        const row: HTMLTableRowElement = document.createElement("tr");

        row.appendChild(createElement("td", { "textContent": team.rank }));

        createLink("team", team.id, team.name, [document.createElement("td"), row]);
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
        // createEventListener(`.team${team.id}`, "click", drawTeamScreen);
    }
};

// Draw a competition schedule.
// Only previous and next matches for now.
const drawSchedule = (screen: HTMLDivElement, season: Season, displaySeed: boolean) => {
    screen.insertAdjacentHTML("beforeend", `
        <table id="previous"><tbody><tr>
            <th id="previous-date"></th>
        </tr></tbody></table>

        <table id="next"><tbody><tr>
            <th id="next-date"></th>
            </tr></tbody></table>
    `);

    drawGameDay("previous", season.played_games, displaySeed);
    drawGameDay("next", season.upcoming_games, displaySeed);
};

// Draw either a past or a future gameday.
// Query can either be "previous" or "next".
const drawGameDay = (query: string, gameList: Array<Game>, displaySeed: boolean) => {
    const table = (document.querySelector(`#${query}`) as HTMLTableElement).children[0] as HTMLTableSectionElement;
    const dateCell = document.querySelector(`#${query}-date`) as HTMLTableCellElement;
    let date = "";

    if (displaySeed) {
        dateCell.colSpan = 5;
    }
    else {
        dateCell.colSpan = 3;
    }

    for (let i = gameList.length - 1; i >= 0; i--) {
        const match = gameList[i];
        if (date === "") { date = match.date; }
        else if (date !== match.date) { break; }

        table.appendChild(drawGame(query, match, displaySeed));
    }

    if (date === "") {
        if (query === "previous") { date = "No previous games."; }
        else { date = "No upcoming games." }
    }
    else {
        if (query === "previous") { date = `Previous games from ${date}.`; }
        else { date = `Next games on ${date}.`; }
    }

    dateCell.textContent = date;
};

// Draw a game row for competition screen schedule.
const drawGame = (query: string, game: Game, displaySeed: boolean) => {
    let scoreString = "-";
    // Add score.
    if (query === "previous") {
        let otString = "";
        if (game.had_overtime) {
            otString = " OT";
        }
        scoreString = `${game.home.goals} ${scoreString}${otString} ${game.away.goals}`;
    }

    const row: HTMLTableRowElement = document.createElement("tr");

    createLink("team", game.home.id, game.home.name, [document.createElement("td"), row]);
    row.appendChild(createElement("td", { "textContent": scoreString }));
    createLink("team", game.away.id, game.away.name, [document.createElement("td"), row]);

    if (displaySeed) {
        row.insertBefore(createElement("td", { "textContent": game.home.seed }), row.firstChild);
        row.appendChild(createElement("td", { "textContent": game.away.seed }));
    }

    return row;
};

// Draw rankings for a competition.
const drawRanking = (screen: HTMLDivElement, teams: Array<Team>) => {
    screen.insertAdjacentHTML("beforeend", `
        <table id="ranking"><tbody><tr>
            <th>Rank</th>
            <th>Team</th>
        </tr></tbody></table>
    `);

    const ranking = (document.querySelector("#ranking") as HTMLTableElement).children[0] as HTMLTableSectionElement;
    for (const team of teams) {
        const row = document.createElement("tr");

        row.appendChild(createElement("td", { "textContent": team.rank }));
        createLink("team", team.id, team.name, [document.createElement("td"), row]);
        ranking.appendChild(row);
    }
};

// Draw pairs of the knockout round.
const drawKnockoutPairs = (screen: HTMLDivElement, pairs: Array<KnockoutPair>) => {
    screen.insertAdjacentHTML("beforeend", `
        <div id="pairs"></div>
    `);

    const pairsElement: HTMLDivElement = document.querySelector("#pairs") as HTMLDivElement;
    createKnockoutPairElements(pairs, pairsElement);
};

// Create elements of knockout pairs and append them to the given parent element.
const createKnockoutPairElements = (pairs: Array<KnockoutPair>, parentElement: HTMLDivElement) => {
    for (const pair of pairs) {
        const table = document.createElement("table");
        const tbody = document.createElement("tbody");
        table.appendChild(tbody);

        drawKnockoutPairTeam(pair.home, tbody);
        drawKnockoutPairTeam(pair.away, tbody);
        parentElement.appendChild(table);
    }
};

const drawKnockoutPairTeam = (team: KnockoutTeam, parentElement: HTMLTableSectionElement) => {
    const row = document.createElement("tr");
    row.appendChild(createElement("td", { "textContent": `${team.seed}.` }));

    createLink("team", team.id, team.name, [document.createElement("td"), row]);
    row.appendChild(createElement("td", { "textContent": team.wins }));

    parentElement.appendChild(row);
}