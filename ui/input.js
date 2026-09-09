const { invoke } = window.__TAURI__.core;
const { getCurrentWindow } = window.__TAURI__.window;
const { listen } = window.__TAURI__.event;

const input = document.getElementById("overlay-input");
const appWindow = getCurrentWindow();

let active = false;

function focusInput() {
    requestAnimationFrame(() => {
        input.focus();
        input.select();
    });
}

function hideOverlay() {
    active = false;
    appWindow.hide().catch(() => {});
}

async function saveAndHide() {
    const text = input.value.trim();
    if (text) {
        try {
            await invoke("park_note", { text });
        } catch (err) {
            console.error("Notiz konnte nicht gespeichert werden:", err);
        }
    }
    input.value = "";
    hideOverlay();
}

input.addEventListener("keydown", (event) => {
    if (event.key === "Enter") {
        event.preventDefault();
        saveAndHide();
    } else if (event.key === "Escape") {
        event.preventDefault();
        hideOverlay();
    }
});

window.addEventListener("focus", () => {
    active = true;
    focusInput();
});

window.addEventListener("blur", () => {
    if (active) {
        hideOverlay();
    }
});

listen("overlay-shown", () => {
    active = true;
    focusInput();
}).catch(() => {
    // Fallback: the window "focus" handler above still focuses the input.
});

focusInput();
