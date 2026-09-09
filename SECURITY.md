VERDICT: CHANGES_REQUESTED

## Sicherheitsprüfung

- **Secrets:** Keine hartkodierten Schlüssel, Passwörter, Tokens oder URLs sichtbar.
- **Injection/Inputs:** Die sichtbaren SQL-Abfragen binden Werte über `rusqlite::params!` als Parameter; keine String-Interpolation in SQL. Die UI rendert Notiztext und Metadaten ausschließlich über `textContent`, nicht über `innerHTML`. Keine XSS- oder SQL-Injection sichtbar.
- **AuthN/AuthZ:** Lokale Einbenutzer-Desktop-App ohne Konto, Server oder Netzwerkzugriff. Keine Authentifizierungs- oder Autorisierungsfunktion erforderlich; keine Befunde.
- **Abhängigkeiten:** Kein verwertbarer Scanner-Output vorhanden; AC-13 kann daher nicht automatisiert verifiziert werden (siehe Hinweise).
- **Konfiguration/Transport:** Keine ausgehenden Netzwerkverbindungen, keine Telemetrie sichtbar. Dateirechte werden restriktiv gesetzt, aber Fehler beim Setzen werden ignoriert (Befund).

## Befunde

### 1. Medium — AC-17: Vollständiges Löschen lässt Daten in der SQLite-Datei zurück  
**Betroffene Stelle:** `src-tauri/src/db.rs`, Funktionen `delete_note` und `delete_all`.  

`delete_all` führt nur `DELETE FROM notes` aus; `delete_note` löscht nur den jeweiligen Datensatz. SQLite überschreibt freigegebene Seiten standardmäßig nicht. Dadurch bleiben Zettel- und Kontextdaten wie Repo-Pfad, Branch, Commit-Hash und geänderte Dateien in der SQLite-Datei potenziell forensisch wiederherstellbar. Das verletzt AC-17 [Datenschutz] und unterläuft auch AC-08 („Das Löschen entfernt sämtliche Daten“).

**Konkrete Behebung:**
- In `open_db` nach dem Öffnen `PRAGMA secure_delete = ON;` setzen.
- Nach `delete_all` zusätzlich `VACUUM;` ausführen, um freigegebene Seiten zu bereinigen; optional `PRAGMA auto_vacuum = FULL`.
- Tests ergänzen, die prüfen, dass nach dem Löschen keine wiederherstellbaren Seiten zurückbleiben.

### 2. Low — AC-10: Fehlgeschlagene Rechtevergabe wird still ignoriert  
**Betroffene Stelle:** `src-tauri/src/db.rs`, `set_restrictive_permissions` / `open_db`.  

Der Ausdruck `let _ = std::fs::set_permissions(...)` verwirft Fehler. Schlägt das Setzen von `0600` fehl, bleibt die Datenbankdatei mit den Standardrechten des Prozesses (häufig `0644`) lesbar. AC-10 ist damit nicht garantiert.

**Konkrete Behebung:**
- Fehler nicht still verwerfen, sondern mindestens deutlich loggen; besser als `Result` aus `open_db` propagieren oder mit `expect`/`panic` fail-closed verfahren, damit die Datei nicht mit unzureichenden Rechten genutzt wird.

## Hinweise (nicht-blockierend)

- Es wurde keine verwertbare Scanner-Ausgabe übermittelt (`no applicable security scanners for this project type`). AC-13 kann daher anhand des vorliegenden Scans nicht abschließend bestätigt werden. Eine gezielte Prüfung der `Cargo.lock` mit `cargo audit` wäre sinnvoll.
- `src-tauri/capabilities/default.json` und `src-tauri/tauri.conf.json` liegen vor, sind hier aber nicht vollständig inspizierbar. Es sollte sichergestellt werden, dass nur die benötigten Tauri-Commands/Permissions freigegeben sind und keine Remote-Inhalte geladen werden.
- `PARKPLATZ_DB_PATH` erlaubt eine beliebige DB-Pfad-Ablage. Unter demselben lokalen Benutzer unkritisch, sollte aber als Test-Naht dokumentiert bleiben.