type QuerySelector = Document["querySelector"];
export type Query = Parameters<QuerySelector>[0];
type QueryResult = ReturnType<QuerySelector>;
export type EventType = Parameters<NonNullable<QueryResult>["addEventListener"]>[0];
export type Listener = Parameters<NonNullable<QueryResult>["addEventListener"]>[1];
type CreateElement = Document["createElement"];
export type TagName = Parameters<CreateElement>[0];

// Types allowed in creating links.
export const LINK_TYPES = ["team", "player"];
export type LinkType = (typeof LINK_TYPES)[number];


export type HumanTeamInfo = {
    id: number,
    actions_remaining: number,
    approached_players: Array<number>
};

// Important info of a human manager.
export type HumanInfo = {
    team: HumanTeamInfo | null
};