// Generic helper functions to alleviate the tedious verbosity.

import { Query, Listener, EventType, TagName, LinkType, LINK_TYPES } from "./types";
import { drawScreen as drawTeamScreen } from "./screens/team";
import { drawScreen as drawPlayerScreen } from "./screens/player";

// Do not touch anything, It Just Works™.
export const createEventListener = (query: Query, event: EventType, listener: Listener) => {
    const elements = document.querySelectorAll(query);
    if (elements.length === 0) {
        console.error(`${query} not found`);
        return;
    }

    for (const element of elements) {
        element.addEventListener(event, listener);
    }
};

// Create an HTML element, give it values that you want and return it.
export const createElement = (elementType: TagName, attributes: any) => {
    const element: any = document.createElement(elementType);
    for (const [key, value] of Object.entries(attributes)) {
        element[key] = value;
    }
    return element;
};


// Draw a link field for any purpose.
export const createLink = (type: LinkType, id: number, name: string, parentElements: Array<HTMLElement>) => {
    const span = createElement("span", { "textContent": name });
    span.classList.add(`${type}${id}`);

    // The event listener needs to be created later.

    for (const [i, element] of parentElements.entries()) {
        if (i === 0) {
            element.appendChild(span);
        }
        else {
            element.appendChild(parentElements[i - 1]);
        }
    }
};

// A dynamic link listener function that makes it possible to create listeners without having the associated element in the document.
export const linkListener: Listener = (e: Event) => {
    const target = e.target;

    // Getting rid of possible bad actors early.
    if (target === null || !("tagName" in target) || target.tagName !== "SPAN") { return; }
    const element = target as HTMLSpanElement;

    for (const linkType of LINK_TYPES) {
        const regex = new RegExp(`${linkType}([0-9]+)`);

        for (const elementClass of element.classList) {
            const match = elementClass.match(regex);
            if (match === null) { continue; }

            const id = Number(match[1]);

            switch (linkType) {
                case "team": {
                    drawTeamScreen(id);
                    return;
                }
                case "player": {
                    drawPlayerScreen(id);
                    return;
                }
                default: {
                    console.error(`Unknown link type ${linkType} with ID ${match[1]}`);
                }
            }
        }
    }
};