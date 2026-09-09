# Parkplatz

Ein Kontext-Merker für Entwickler als Tray-App. Parkplatz läuft unsichtbar im
System-Tray und erlaubt es, den aktuellen Arbeitskontext — Repository-Pfad,
Branch, letzter Commit, geänderte Dateien und eine Zeile Freitext — per globalem
Hotkey als „Zettel" abzulegen. Beim erneuten Wechsel auf einen bereits
geparkten Branch erscheint der passende Zettel als unaufdringliche
Benachrichtigung. Alle Daten bleiben lokal in einer SQLite-Datei; es gibt kein
Konto, keinen Server und keinen Netzwerkzugriff.

## Tech-Stack

- **Sprache**: Rust
- **GUI/Framework**: Tauri v2 (Tray, globaler Hotkey, Overlay-Fenster)
- **Datenbank**: SQLite via `rusqlite`
- **Git-Zustand**: libgit2 (`git2`)
- **Dateisystem-Watcher**: `notify`
- **Plattform**: Linux zuerst, macOS und Windows als weitere Zielplattformen

## Installation

Voraussetzungen: eine aktuelle Rust-Toolchain (`rustup`), sowie die
systemseitigen Abhängigkeiten für Tauri (siehe Bauhinweise unten).

```bash
cargo install tauri-cli --locked
```

Die Projektabhängigkeiten werden beim ersten Build automatisch über Cargo
aufgelöst (`rusqlite` wird in der Variante `bundled` gebaut und benötigt daher
kein systemweites SQLite).

## Starten (Entwicklung)

```bash
cd src-tauri
cargo tauri dev
```

Das startet die App im Vordergrund: ein Tray-Icon erscheint, das Hauptfenster
mit dem Titel „Parkplatz" öffnet sich, und das Tray-Menü bietet „Öffnen" und
„Beenden".

## Bauen (Produktion)

```bash
cd src-tauri
cargo tauri build
```

Das Ergebnis liegt unter `src-tauri/target/release/` bzw. als gebündeltes Paket
(z. B. `.deb`, `.AppImage`, `.dmg`, `.msi`) in
`src-tauri/target/release/bundle/`.

## Bedienung

- **Tray-Icon**: linke Maustaste öffnet das Menü mit den Einträgen **Öffnen**
  (zeigt das Hauptfenster) und **Beenden** (beendet die App).
- **Hauptfenster**: zeigt den Titel „Parkplatz", ein Schnell-Eingabefeld, ein
  Suchfeld sowie die (zunächst leere) Zettelliste mit Schaltflächen zum
  Exportieren und zum Löschen aller Zettel.
- **Overlay-Fenster**: ein separates, verstecktes Eingabefenster
  (`ui/input.html`), das später über den globalen Hotkey geöffnet wird.

## Features

- Tray-App mit Tray-Icon und Menü (Öffnen / Beenden)
- Hauptfenster mit Suchfeld, Schnell-Eingabefeld und Zettelliste
- Datenmodell für Notizen (`Note`) und Git-Kontext (`GitContext`)
- Tauri-Befehle für Parken, Auflisten, Suchen, Abhaken, Löschen, JSON-Export und
  Git-Kontext-Abfrage (in diesem Grundgerüst als Gerüst verdrahtet)

## Bauhinweise je Plattform

- **Linux**: benötigt `webkit2gtk-4.1` bzw. `webkit2gtk-4.0` sowie die üblichen
  Build-Tools (`build-essential`, `libssl-dev`, `libgtk-3-dev`,
  `libayatana-appindicator3-dev`, `librsvg2-dev`). Unter Debian/Ubuntu z. B.:
  `sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev`.
- **macOS**: benötigt die Xcode Command Line Tools (`xcode-select --install`).
- **Windows**: benötigt Microsoft C++ Build Tools und das WebView2-Runtime;
  ein natives Tray-Icon wird über den in `tauri.conf.json` referenzierten
  `icon.ico` bereitgestellt.
