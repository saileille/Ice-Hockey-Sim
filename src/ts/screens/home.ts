// The main screen of the game.

import { createEventListener } from "../helpers";
import { Listener } from "../types";
import { initialiseContentScreen, initialiseTopBar } from "./basics";
import { drawScreen as drawPlayerSearchScreen } from "./player_search";

export const drawScreen = () => {
    initialiseTopBar();

    // Do stuff with this later.
    const screen = initialiseContentScreen();
    screen.innerHTML = `
        <button id="player-search">Free Agents</button>
    `;

    createEventListener("#player-search", "click", drawPlayerSearchScreen);
};

// Listener of home screen button.
export const onClickHomeScreen: Listener = (_e: Event) => {
    drawScreen();
}