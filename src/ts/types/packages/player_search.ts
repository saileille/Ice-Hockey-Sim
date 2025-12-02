export type Player = {
    person: Person,
    position: string,
    ability: number,
};

type Person = {
    id: number,
    full_name: string,
    country: Country,
    age: number,
    no_of_offers: number,
}

type Country = {
    name: string,
    flag_path: string,
};