type QuerySelector = Document["querySelector"];

type Query = Parameters<QuerySelector>[0];
type QueryResult = ReturnType<QuerySelector>;

type EventType = Parameters<NonNullable<QueryResult>["addEventListener"]>[0];
export type Listener = Parameters<NonNullable<QueryResult>["addEventListener"]>[1];

type CreateElement = Document["createElement"];
type TagName = Parameters<CreateElement>[0];

// A generic helper function to alleviate the tedious verbosity.
// Do not touch anything, It Just Works™.
export const createEventListener = (query: Query, event: EventType, listener: Listener) => {
    document.querySelector(query)?.addEventListener(event, listener)
};

// Create an HTML element, give it values that you want and return it.
export const createElement = (elementType: TagName, attributes: any) => {
    let element: any = document.createElement(elementType);
    for (const [key, value] of Object.entries(attributes)) {
        element[key] = value;
    }
    return element;
};

/* const greetInputEl: HTMLInputElement | null;
const greetMsgEl: HTMLElement | null; */

/*const greet = async () => {
    if (greetMsgEl && greetInputEl) {
        // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
        greetMsgEl.textContent = await invoke("greet", {
            name: greetInputEl.value,
        });
    }
};*/

/* window.addEventListener("DOMContentLoaded", () => {
    greetInputEl = document.querySelector("#greet-input");
    greetMsgEl = document.querySelector("#greet-msg");
    document.querySelector("#greet-form")?.addEventListener("submit", (e) => {
        e.preventDefault();
        // testGame();
        testComp();
    });
}); */