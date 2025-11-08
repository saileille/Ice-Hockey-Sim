type QuerySelector = Document["querySelector"];
export type Query = Parameters<QuerySelector>[0];
type QueryResult = ReturnType<QuerySelector>;
export type EventType = Parameters<NonNullable<QueryResult>["addEventListener"]>[0];
export type Listener = Parameters<NonNullable<QueryResult>["addEventListener"]>[1];
type CreateElement = Document["createElement"];
export type TagName = Parameters<CreateElement>[0];

// Types allowed in creating links.
export const LINK_TYPES = ["team", "player", "comp"];
export type LinkType = (typeof LINK_TYPES)[number];


export type Position = "GK" | "LD" | "RD" | "LW" | "C" | "RW";

export type RosterOverview = Array<{
    "position": Position,
    "id": number,
    "in_roster": boolean,
}>;

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