// Show the current date.
import { invoke } from "@tauri-apps/api/core";

const displayDate = async () => {
    let dateDiv = document.querySelector("#date");
    if (dateDiv === null) { return; }

    let dateString: string = await invoke("get_date_string");
    dateDiv.textContent = dateString;
};

displayDate();