// Generic helper functions to alleviate the tedious verbosity.
import { convertFileSrc } from "@tauri-apps/api/core";

import { drawScreen as drawTeamScreen } from "./screens/team";
import { drawScreen as drawPlayerScreen } from "./screens/player";
import { drawScreen as drawCompScreen } from "./screens/competition";
import { EventType, LinkType, Listener, Query, TagName } from "./types/dom";
import { CountryNameAndFlag } from "./types/team";

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
export const createElement = (elementType: TagName, attributes: any, children: Array<Element | string>) => {
    const element: any = document.createElement(elementType);
    for (const [key, value] of Object.entries(attributes)) {
        element[key] = value;
    }

    element.append(...children);
    return element;
};


// Draw a link field for any purpose.
export const createLink = (tag: string, type: LinkType, id: number, text: string): HTMLElement => {
    return createElement(tag, {
        "textContent": text,
        "className": `${type}${id} link`
    }, []);
};

export const createImage = (country: CountryNameAndFlag, type: "block" | "inline"): HTMLSpanElement | HTMLImageElement => {
    if (country.flag_path === null) {
        return createElement("span", {
            "textContent": country.name,
        }, [])
    };

    return createElement("img", {
        "className": `contained-${type}`,    // Using this to resize after drawing.
        "src": convertFileSrc(country.flag_path),
        "title": country.name,
    }, [])
};

// Resize all graphical elements at once. Can be used for other images as well.
export const resizeImages = () => {
    const images = document.querySelectorAll("img.contained-block, img.contained-inline") as NodeListOf<HTMLImageElement>;
    console.log(images.length);
    for (const image of images) {
        const height = calculateImageHeight(getComputedStyle(image.parentElement as HTMLElement));
        console.log(height);
        image.style.height = height;
    }
};

// Get the actual height of the graphics element, or die trying.
const calculateImageHeight = (computed: CSSStyleProperties): string => {
    let height = parseFloat(computed.getPropertyValue("line-height"));
    return `${height}px`;
};

// A dynamic link listener function that makes it possible to create listeners without having the associated element in the document.
export const linkListener: Listener = (e: Event) => {
    const element = e.target as Element;

    // Getting rid of possible bad actors early.
    if (element === null || !element.classList.contains("link")) { return; }

    // Getting the link type and the ID from the element.
    const link = extractIdFromElement(element);
    // console.dir(link);

    if (link === undefined) { return; }

    switch (link[0]) {
        case "comp": {
            drawCompScreen(link[1]);
            return;
        }
        case "team": {
            drawTeamScreen(link[1]);
            return;
        }
        case "player": {
            drawPlayerScreen(link[1]);
            return;
        }
        default: {
            console.error(`Unknown link type ${link[0]} with ID ${link[1]}`);
        }
    }
};

// Extract the ID of a data item from DOM element ID.
export const extractIdFromElement = (element: Element): [LinkType, number] | undefined => {
    const regex = new RegExp(`([a-z]+)([0-9]+)`);

    for (const className of element.classList) {
        const match = className.match(regex);
        if (match === null) { continue; }

        return [match[1], Number(match[2])];
    }
};