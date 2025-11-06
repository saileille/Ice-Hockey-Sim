// The main screen of the game.

import { Listener } from "../types";
import { initialiseContentScreen } from "./basics";

export const drawScreen = () => {
    const screen = initialiseContentScreen();
};

// Listener of home screen button.
export const onClickHomeScreen: Listener = (_e: Event) => {
    drawScreen();
};