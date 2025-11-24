// Draw the competition screen of the competition given in the ID.
import { invoke } from "@tauri-apps/api/core";
import { initialiseContentScreen, createCompNav } from "./basics";
import { createElement, createLink } from "../helpers";

type Format = {
    round_robin: RoundRobinFormat | null,
    knockout_round: KnockoutRoundFormat | null,
    match_rules: MatchRules,
    type: string
};

type CompetitionType = "Null" | "Tournament";

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
    comp_nav: Array<Array<[number, string]>>,
    competition_type: CompetitionType
};

// Draw any competition screen.
export const drawScreen = async (id: number) => {
    const comp: Competition = await invoke("comp_screen_package", { id: id });

    const screen = initialiseContentScreen();

    // If the competition is something like playoffs.
    if (comp.season.rounds !== undefined) {
        drawScreenTournament(screen, comp, comp.season.rounds);
    }

    else if (comp.format === null) {
        drawScreenParent(screen, comp);
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
const drawScreenTournament = (screen: HTMLDivElement, comp: Competition, rounds: Array<KnockoutRound>) => {
    createCompNav(screen, comp.comp_nav);

    screen.append(
        drawTournamentTree(rounds),
        drawSchedule(comp.season, true)
    );

};

// Draw a tournament tree for a round-robin competition.
const drawTournamentTree = (rounds: Array<KnockoutRound>): HTMLTableElement => {
    const row = document.createElement("tr");
    for (const round of rounds) {
        row.appendChild(
            createElement("td", {}, [
                drawRoundPairs(round.pairs)
            ])
        );
    }

    return createElement("table", {}, [
        createElement("tbody", {}, [row]),
    ]);
};

// Draw a screen for parent competitions.
const drawScreenParent = (screen: HTMLDivElement, comp: Competition) => {
    createCompNav(screen, comp.comp_nav);
    screen.appendChild(drawRanking(comp.season.teams));
};

// Draw a screen for round robin competitions.
const drawScreenRoundRobin = (screen: HTMLDivElement, comp: Competition) => {
    createCompNav(screen, comp.comp_nav);

    screen.append(
        drawRoundRobinStandings(comp.season.teams),
        drawSchedule(comp.season, false)
    );
};

// Draw a screen for a knockout round.
const drawScreenKnockoutRound = (screen: HTMLDivElement, comp: Competition) => {
    createCompNav(screen, comp.comp_nav);

    screen.append(
        drawRoundPairs((comp.season.knockout_round as KnockoutRound).pairs),
        drawSchedule(comp.season, true)
    );
};

// Draw the standings for round robin.
const drawRoundRobinStandings = (teams: Array<Team>): HTMLTableElement => {
    const table = createElement("table", {}, [
        createElement("thead", {}, [
            createElement("tr", {}, [
                createElement("th", { "textContent": "Rank" }, []),
                createElement("th", { "textContent": "Name" }, []),
                createElement("th", { "textContent": "GP" }, []),
                createElement("th", { "textContent": "W" }, []),
                createElement("th", { "textContent": "OTW" }, []),
                createElement("th", { "textContent": "D" }, []),
                createElement("th", { "textContent": "OTL" }, []),
                createElement("th", { "textContent": "L" }, []),
                createElement("th", { "textContent": "GF" }, []),
                createElement("th", { "textContent": "GA" }, []),
                createElement("th", { "textContent": "Diff" }, []),
                createElement("th", { "textContent": "Pts." }, []),
            ])
        ])
    ]);

    const standings = document.createElement("tbody");
    for (const team of teams) {
        standings.appendChild(createElement("tr", {}, [
            createElement("td", { "textContent": team.rank }, []),
            createElement("td", {}, [createLink("span", "team", team.id, team.name)]),
            createElement("td", { "textContent": team.games }, []),
            createElement("td", { "textContent": team.wins }, []),
            createElement("td", { "textContent": team.ot_wins }, []),
            createElement("td", { "textContent": team.draws }, []),
            createElement("td", { "textContent": team.ot_losses }, []),
            createElement("td", { "textContent": team.losses }, []),
            createElement("td", { "textContent": team.goals_scored }, []),
            createElement("td", { "textContent": team.goals_conceded }, []),
            createElement("td", { "textContent": team.goal_difference }, []),
            createElement("td", { "textContent": team.points }, []),
        ]));
    }

    table.appendChild(standings);
    return table;
};

// Draw a competition schedule.
// Only previous and next matches for now.
const drawSchedule = (season: Season, displaySeed: boolean): HTMLTableElement => {
    return createElement("table", {}, [
        createElement("tbody", {}, [
            createElement("tr", {}, [
                drawGameDay(true, season.played_games, displaySeed),
                drawGameDay(false, season.upcoming_games, displaySeed)
            ])
        ])
    ]);
};

// Draw either a past or a future gameday.
// Query can either be "previous" or "next".
const drawGameDay = (isPast: boolean, gameList: Array<Game>, displaySeed: boolean): HTMLTableCellElement => {
    const tbody = document.createElement("tbody");
    let date = "";

    let dateColumns = 3;
    if (displaySeed) dateColumns = 5;

    const matches = [];
    for (let i = gameList.length - 1; i >= 0; i--) {
        const match = gameList[i];
        if (date === "") { date = match.date; }
        else if (date !== match.date) { break; }

        matches.push(drawGame(isPast, match, displaySeed));
    }

    // We need to reverse the games if the date is in the past.
    if (isPast) matches.reverse();
    tbody.append(...matches);

    if (date === "") {
        if (isPast) { date = "No previous games."; }
        else { date = "No upcoming games." }
    }
    else {
        if (isPast) { date = `Previous games from ${date}.`; }
        else { date = `Next games on ${date}.`; }
    }

    return createElement("td", { "style": "vertical-align: top;" }, [
        createElement("table", {}, [
            createElement("thead", {}, [
                createElement("tr", {}, [
                    createElement("th", {
                        "textContent": date,
                        "colSpan": dateColumns,
                    }, [])
                ])
            ]),
            tbody
        ])
    ]);
};

// Draw a game row for competition screen schedule.
const drawGame = (isPast: boolean, game: Game, displaySeed: boolean) => {
    const row = document.createElement("tr");
    if (displaySeed) {
        row.appendChild(createElement("td", { "textContent": `(${game.home.seed}.)` }, []));
    }

    row.append(
        createElement("td", {}, [createLink("span", "team", game.home.id, game.home.name)]),
        createElement("td", { "textContent": getScoreString(isPast, game) }, []),
        createElement("td", {}, [createLink("span", "team", game.away.id, game.away.name)]),
    );

    if (displaySeed) {
        row.appendChild(createElement("td", { "textContent": `(${game.away.seed}.)` }, []));
    }

    return row;
};

// Return the score string for a game.
const getScoreString = (isPast: boolean, game: Game) => {
    let scoreString = "-";
    if (isPast) {
        let otString = "";
        if (game.had_overtime) {
            otString = " OT";
        }
        scoreString = `${game.home.goals} ${scoreString} ${game.away.goals}${otString}`;
    }
    return scoreString;
};

// Draw rankings for a competition.
const drawRanking = (teams: Array<Team>): HTMLTableElement => {
    const tbody = createElement("tbody", {}, [
        createElement("tr", {}, [
            createElement("th", { "textContent": "Rank" }, []),
            createElement("th", { "textContent": "Team" }, []),
        ])
    ]);

    for (const team of teams) {
        tbody.appendChild(createElement("tr", {}, [
            createElement("td", { "textContent": team.rank }, []),
            createElement("td", {}, [createLink("span", "team", team.id, team.name)]),
        ]));
    }

    return createElement("table", {}, [tbody]);
};

// Draw the pairs of a knockout round.
const drawRoundPairs = (pairs: Array<KnockoutPair>): HTMLTableElement => {
    const tbody = document.createElement("tbody");
    for (const pair of pairs) {
        tbody.append(
            drawKnockoutPairTeam(pair.home),
            drawKnockoutPairTeam(pair.away)
        );
    }

    return createElement("table", {}, [tbody]);
};

const drawKnockoutPairTeam = (team: KnockoutTeam): HTMLTableRowElement => {
    return createElement("tr", {}, [
        createElement("td", { "textContent": `${team.seed}.` }, []),
        createElement("td", {}, [createLink("span", "team", team.id, team.name)]),
        createElement("td", { "textContent": `${team.wins}` }, []),
    ]);
}