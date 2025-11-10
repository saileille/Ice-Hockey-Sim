import { Player, Position } from "./player";

export type RosterOverview = Array<{
    "position": Position,
    "id": number,
    "in_roster": boolean,
}>;

type Manager = {
    name: string
};


export type HumanTeamPackage = {
    id: number,
    actions_remaining: number,

    // Roster overview includes approached players as well.
    // This is dupliocated information, but makes accessing it much easier.
    roster_overview: RosterOverview,
    approached_players: Array<number>,
};

// Important info of a human manager.
export type HumanPackage = {
    team: HumanTeamPackage | null
};
export type RosterSetting = "roster" | "approached" | "both";

export type Team = {
    id: number,
    name: string,
    manager: Manager | null,
    players: Array<Player>
};

export type TopBarPackage = {
    "date": string,
    "human": HumanPackage,
};