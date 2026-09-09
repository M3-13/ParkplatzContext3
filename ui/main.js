const { invoke } = window.__TAURI__.core;

const listEl = document.getElementById("notes-list");
const searchInput = document.getElementById("search-input");
const quickInput = document.getElementById("quick-input");
const quickSaveBtn = document.getElementById("quick-save");
const exportBtn = document.getElementById("export-btn");
const deleteAllBtn = document.getElementById("delete-all-btn");

function injectStyles() {
    if (document.getElementById("parkplatz-note-styles")) return;
    const style = document.createElement("style");
    style.id = "parkplatz-note-styles";
    style.textContent = `
        .note-group { margin-bottom: var(--space-4, 16px); }
        .note-group-title {
            margin: 0 0 var(--space-2, 8px);
            font-size: 14px;
            font-weight: 600;
            color: var(--color-text-muted, #6b7484);
            overflow-wrap: anywhere;
        }
        .note-card {
            padding: var(--space-3, 12px);
            margin-bottom: var(--space-2, 8px);
            border: 1px solid var(--color-border, #d9dee7);
            border-radius: var(--radius-md, 8px);
            background-color: var(--color-surface, #ffffff);
        }
        .note-card.note-done { opacity: 0.6; }
        .note-card.note-done .note-text { text-decoration: line-through; }
        .note-text { margin: 0 0 var(--space-2, 8px); overflow-wrap: anywhere; white-space: pre-wrap; }
        .note-meta {
            display: flex;
            flex-wrap: wrap;
            gap: var(--space-2, 8px);
            margin-bottom: var(--space-2, 8px);
            font-size: 12px;
            color: var(--color-text-muted, #6b7484);
            overflow-wrap: anywhere;
        }
        .note-actions { display: flex; gap: var(--space-2, 8px); align-items: center; }
        .note-check { display: inline-flex; align-items: center; gap: var(--space-1, 4px); cursor: pointer; }
    `;
    document.head.appendChild(style);
}

function el(tag, className, text) {
    const node = document.createElement(tag);
    if (className) node.className = className;
    if (text !== undefined && text !== null) node.textContent = text;
    return node;
}

function formatDate(createdAt) {
    if (!createdAt) return "";
    const parsed = new Date(createdAt);
    if (Number.isNaN(parsed.getTime())) return createdAt;
    return parsed.toLocaleString();
}

function renderNote(note) {
    const card = el("article", "note-card" + (note.done ? " note-done" : ""));

    card.appendChild(el("p", "note-text", note.text));

    const meta = el("div", "note-meta");
    if (note.branch) meta.appendChild(el("span", "note-branch", note.branch));
    if (note.commit_hash) meta.appendChild(el("span", "note-hash", note.commit_hash));
    if (note.changed_files && note.changed_files.length) {
        meta.appendChild(el("span", "note-files", note.changed_files.join(", ")));
    }
    meta.appendChild(el("span", "note-date", formatDate(note.created_at)));
    card.appendChild(meta);

    const actions = el("div", "note-actions");

    const check = el("label", "note-check");
    const checkbox = document.createElement("input");
    checkbox.type = "checkbox";
    checkbox.checked = Boolean(note.done);
    checkbox.addEventListener("change", () => toggleDone(note, checkbox.checked));
    check.appendChild(checkbox);
    check.appendChild(el("span", "", note.done ? "Abgehakt" : "Abhaken"));
    actions.appendChild(check);

    const deleteBtn = el("button", "btn note-delete-btn", "Löschen");
    deleteBtn.type = "button";
    deleteBtn.addEventListener("click", () => removeNote(note));
    actions.appendChild(deleteBtn);

    card.appendChild(actions);
    return card;
}

function renderNotes(notes) {
    listEl.replaceChildren();
    if (!notes || notes.length === 0) {
        listEl.appendChild(el("p", "empty-state", "Noch keine Zettel vorhanden."));
        return;
    }

    const groups = new Map();
    for (const note of notes) {
        const repo = note.repo_path || "(kein Repo)";
        if (!groups.has(repo)) groups.set(repo, []);
        groups.get(repo).push(note);
    }

    for (const [repo, repoNotes] of groups) {
        const group = el("section", "note-group");
        group.appendChild(el("h2", "note-group-title", repo));
        for (const note of repoNotes) group.appendChild(renderNote(note));
        listEl.appendChild(group);
    }
}

async function loadNotes() {
    try {
        const notes = await invoke("list_notes");
        renderNotes(notes);
    } catch (err) {
        console.error("Zettel konnten nicht geladen werden.", err);
    }
}

async function search() {
    const query = searchInput.value.trim();
    try {
        const notes = query
            ? await invoke("search_notes", { query })
            : await invoke("list_notes");
        renderNotes(notes);
    } catch (err) {
        console.error("Suche fehlgeschlagen.", err);
    }
}

async function quickPark() {
    const text = quickInput.value.trim();
    if (!text) return;
    try {
        await invoke("park_note", { text });
        quickInput.value = "";
        await loadNotes();
    } catch (err) {
        console.error("Zettel konnte nicht angelegt werden.", err);
    }
}

async function toggleDone(note, done) {
    try {
        await invoke("set_note_done", { id: note.id, done });
        note.done = done;
        await loadNotes();
    } catch (err) {
        console.error("Abhaken fehlgeschlagen.", err);
    }
}

async function removeNote(note) {
    try {
        await invoke("delete_note", { id: note.id });
        await loadNotes();
    } catch (err) {
        console.error("Löschen fehlgeschlagen.", err);
    }
}

async function exportNotes() {
    try {
        const json = await invoke("export_notes_json");
        const blob = new Blob([json], { type: "application/json" });
        const url = URL.createObjectURL(blob);
        const anchor = document.createElement("a");
        anchor.href = url;
        anchor.download = "parkplatz-notizen.json";
        document.body.appendChild(anchor);
        anchor.click();
        document.body.removeChild(anchor);
        URL.revokeObjectURL(url);
    } catch (err) {
        console.error("Export fehlgeschlagen.", err);
    }
}

async function deleteAllNotes() {
    if (!window.confirm("Wirklich alle Zettel löschen?")) return;
    try {
        await invoke("delete_all_notes");
        await loadNotes();
    } catch (err) {
        console.error("Alles-löschen fehlgeschlagen.", err);
    }
}

injectStyles();

searchInput.addEventListener("input", search);
quickSaveBtn.addEventListener("click", quickPark);
quickInput.addEventListener("keydown", (event) => {
    if (event.key === "Enter") quickPark();
});
exportBtn.addEventListener("click", exportNotes);
deleteAllBtn.addEventListener("click", deleteAllNotes);

loadNotes();
