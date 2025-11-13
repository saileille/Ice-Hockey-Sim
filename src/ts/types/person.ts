import { CountryNameAndFlag } from "./team";

type Person = {
    name: string,
    country: CountryNameAndFlag,
    age: number,
    birthday: string,
    contract: Contract | null,
    offers: Array<Contract>,
};

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
    person: Person,
    id: number,
    position: Position,
    ability: number,
    real_ability: number,
};

export type Manager = {
    person: Person,
};