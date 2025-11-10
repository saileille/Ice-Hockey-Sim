export type Position = "GK" | "LD" | "RD" | "LW" | "C" | "RW";

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

export type Player = {
    id: number,
    name: string,
    country: string,
    position: Position,
    age: number,
    birthday: string,
    ability: number,
    real_ability: number,
    contract: Contract | null,
    offers: Array<Contract>
};