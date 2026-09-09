VERDICT: CHANGES_REQUESTED

## Ergebnisüberblick

Der sichtbare Produktstand ist im Kern solide: SQL-Werte werden durchgängig per Parameter gebunden (AC-12), Freitext wird ausschließlich über `textContent` gerendert und nicht als HTML interpretiert (AC-11), es sind keine ausgehenden Netzwerkverbindungen oder Telemetrie-Aufrufe im Code sichtbar (AC-15), und die Log-/Fehlerausgaben enthalten keine Zettel-, Repo-, Branch- oder Commit-Hash-Daten (AC-16). Das Produkt hat kein KI-Modul, der EU AI Act ist daher nicht einschlägig.

Es gibt jedoch einen konkreten, behebbaren Datenschutz-/Security-Mangel bei der Anlage der SQLite-Datei: Die Datei wird nicht bereits mit 0600-Rechten **angelegt**, sondern erst nach dem Öffnen per `set_permissions` nachgezogen — mit einem unsicheren Zeitfenster und still verschlucktem Fehlschlag. Das verletzt AC-10 und AC-14 und führt zum Verdict CHANGES_REQUESTED.

---

## Datenschutz (DSGVO)

### F1 — SQLite-Datei wird nicht von Anfang an mit `0600` angelegt (AC-10, AC-14)

**Schwere:** mittel

**Befund:**
In `src-tauri/src/db.rs`, Funktion `open_db()`, öffnet `Connection::open(&path)` die SQLite-Datei zunächst mit den Standard-Dateirechten (auf Unix typischerweise `0644` abhängig von der umask). Erst danach wird `set_restrictive_permissions` aufgerufen. Damit existiert ein Zeitfenster, in dem andere lokale Benutzer des Systems die Datei lesen können. Zusätzlich wird ein Fehlschlag der Rechtevergabe mit `let _ =` kommentarlos ignoriert; schlägt `set_permissions` fehl, verbleibt die Datei dauerhaft mit zu weit gehenden Rechten.

AC-10 und AC-14 verlangen ausdrücklich, dass die Datei **mit Dateirechten angelegt** wird, die nur dem aktuellen Benutzer Lese- und Schreibzugriff erlauben (Unix: `0600`). Das aktuelle Vorgehen erfüllt die Formulierung nicht vollständig.

**Konkrete Abhilfe:**

In `src-tauri/src/db.rs`, vor `Connection::open`, die Datei bereits mit restriktiven Rechten anlegen, z. B.:

```rust
#[cfg(unix)]
{
    use std::fs::OpenOptions;
    use std::os::unix::fs::OpenOptionsExt;
    // Datei, falls nicht vorhanden, sofort mit 0600 erzeugen
    if !path.exists() {
        let _ = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&path)
            .expect("failed to create database file with 0600");
    }
    // Bestehende Datei strikt nachziehen und Fehler nicht still verschlucken
    if let Err(err) = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)) {
        eprintln!("failed to set restrictive permissions on {}: {err}", path.display());
        // Behandlungsstrategie wählen: z. B. Panik/Fehler zurückgeben,
        // jedenfalls nicht kommentarlos fortfahren.
    }
}
```

Zusätzlich sollte das Elternverzeichnis `data_dir().join("parkplatz")` mit `0700` angelegt werden, damit auch der Pfad selbst nicht für andere Benutzer einsehbar ist. Das ist nicht ausdrücklich durch AC-10/AC-14 gefordert, aber eine naheliegende Absicherung.

---

### H1 — Physisches Löschen in SQLite (AC-17) — Hinweis, nicht blockierend

**Schwere:** gering

**Befund:**
`delete_note()` und `delete_all()` in `src-tauri/src/db.rs` führen lediglich `DELETE FROM notes` aus. SQLite gibt den Speicherplatz dadurch logisch frei, überschreibt die freigegebenen Seiten aber nicht sofort. Forensische Reste der gelöschten Freitexte, Repo-Pfade oder Branch-Namen können in der Datenbankdatei verbleiben, bis die Seiten neu beschrieben oder die Datenbank kompaktiert wird.

AC-17 verlangt „vollständig entfernt … ohne verbleibende Kopien im User-Verzeichnis“. Die logische Entfernung der Einträge ist erfüllt; für eine besonders strenge Auslegung wäre physisches Löschen nötig.

**Optional zur Abhilfe:**

```sql
PRAGMA secure_delete = ON;
```

oder nach `delete_all()` ein

```sql
VACUUM;
```

ergänzen. Beide Maßnahmen haben Performance-Auswirkungen, daher bewusst als Hinweis und nicht als geforderter Fix eingeordnet.

---

### H2 — Unisolierter Test `park_note_builds_a_note_with_text_and_timestamp` schreibt potenziell in die echte Benutzerdatenbank

**Schwere:** gering

**Befund:**
`src-tauri/tests/skeleton_test.rs` ruft `commands::park_note("Hallo Welt")` ohne vorher `PARKPLATZ_DB_PATH` auf ein temporäres Verzeichnis zu setzen. Dadurch kann der Test in einer Entwicklungs- oder CI-Umgebung die echte Produktdatenbank unter `$XDG_DATA_HOME/parkplatz/notes.db` bzw. `%APPDATA%` anlegen oder verändern.

Das ist kein Produktfehler, kann aber auf Entwicklungsrechnern echte lokale Nutzerdaten beeinflussen und dort Datenschutz- bzw. Datenverlustrisiken erzeugen.

**Abhilfe:**
Den Test wie die übrigen DB-Tests isolieren: temporäres Verzeichnis anlegen, `PARKPLATZ_DB_PATH` setzen und die Umgebungsvariable nach dem Test wieder bereinigen. Der vorhandene `temp_db_dir`/`use_db`-Ansatz aus `db_test.rs` ist dafür das passende Muster.

---

## EU Cyber Resilience Act (CRA)

### H3 — AC-13 (keine Abhängigkeiten mit bekannter kritischer CVE) ist im vorliegenden Stand nicht verifizierbar

**Schwere:** gering

**Befund:**
`src-tauri/Cargo.lock` existiert und ist 5485 Zeilen lang, sein Inhalt ist hier aber nicht ausgewertet. Damit kann nicht bestätigt werden, dass keine Abhängigkeit mit einer bekannten kritischen CVE ausgeliefert wird.

AC-13 ist ein Security-Kriterium. Es liegt derzeit kein sichtbarer Verstoß vor, aber auch kein Nachweis der Erfüllung.

**Abhilfe:**
Vor jeder Auslieferung `cargo audit` bzw. einen vergleichbaren CVE-Scan über `Cargo.lock` laufen lassen und das Ergebnis dokumentieren. Bei einem CI-gestützten Projekt empfiehlt sich der Audit als fester Prüfschritt.

---

### H4 — Kein sichtbarer Update-/Patch-Mechanismus und kein SBOM

**Schwere:** gering (nicht blockierend, da kein AC-Kriterium betroffen)

**Befund:**
Die CRA verlangt für Produkte mit digitalen Elementen unter anderem:
- Sicherheit „by design and by default“,
- die Möglichkeit, Sicherheitsupdates bereitzustellen,
- eine SBOM (Software Bill of Materials),
- dokumentierte Sicherheitseigenschaften.

Im sichtbaren Code ist kein Update-Mechanismus vorhanden. Ein SBOM ist nicht abgelegt oder referenziert. Eine Update-Funktion wäre für eine vollständig offline arbeitende Notiz-App technisch nur eingeschränkt sinnvoll, sollte aber für eine spätere Verbreitung zumindest als dokumentierte Architekturentscheidung festgehalten werden.

**Abhilfe:**
In `README.md` oder einem separaten `SECURITY.md` dokumentieren:
- Sicherheitsmodell der App (lokal, keine Netzwerkverbindungen, `0600`-Dateirechte, parametrisierte SQL-Abfragen, Text-Rendering ohne HTML),
- Prozess für Sicherheitsupdates,
- Erzeugung einer SBOM, z. B. mit `cargo cyclonedx` oder `cargo sbom`.

---

## EU AI Act

Kein KI-Modul oder automatisierte Entscheidungsfunktion im Produkt sichtbar. Der EU AI Act ist für dieses Projekt nicht einschlägig. Keine Befunde.

---

## Pflichttexte & UI

### H5 — Kein Impressum / keine Datenschutzerklärung enthalten

**Schwere:** gering (nicht blockierend)

**Befund:**
Die App ist eine lokale Desktop-Anwendung ohne Konto, Server oder Netzwerkzugriff. Eine klassische Impressumspflicht für eine reine Offline-Software besteht in Deutschland regelmäßig nicht. Eine Datenschutzerklärung kann erforderlich werden, sobald die App über App-Stores vertrieben wird oder ein Hersteller-Onlineangebot mit ihr verknüpft ist.

Im Code und in den sichtbaren UI-Dateien (`ui/index.html`, `ui/input.html`) gibt es keine Hinweise auf eine Datenschutzerklärung, Nutzungsbedingungen oder ein Impressum.

**Abhilfe:**
Vor einer Veröffentlichung über Linux-Paketmanager, App-Stores oder die geplanten macOS-/Windows-Kanäle (AC-09) prüfen, ob eine kurze Datenschutzerklärung und ggf. ein Impressum mitgeliefert werden müssen. Diese könnten als statische Textseite im Hauptfenster oder als `README.md`-Abschnitt ergänzt werden. Kein Cookie- oder Consent-Banner nötig, da keine Cookies gesetzt oder Daten übermittelt werden.

---

## Barrierefreiheit

### H6 — Eingabefelder ohne zugehörige Labels

**Schwere:** gering (nicht blockierend, da kein AC-Kriterium betroffen)

**Befund:**
In `ui/index.html` haben das Schnelleingabe-Feld `#quick-input` und das Suchfeld `#search-input` ausschließlich `placeholder`-Texte, aber keine `<label>`-Elemente oder `aria-label`-Attribute. Gleiches gilt für `#overlay-input` in `ui/input.html`. Für Screenreader-Nutzende ist der Zweck der Felder damit nicht eindeutig verfügbar.

Das widerspricht dem Grundsatz der Zugänglichkeit (WCAG/BITV/EAA) und ist unabhängig davon relevant, ob das Produkt als Desktop-App oder WebView-Anwendung eingeordnet wird.

**Abhilfe:**
Jedes Eingabefeld mit `<label for="…">` versehen oder mindestens `aria-label="Notiz eingeben"` bzw. `aria-label="Zettel durchsuchen"` ergänzen. Beispiel:

```html
<label for="quick-input" class="sr-only">Notiz eingeben</label>
<input type="text" id="quick-input" … />
```

---

## Nicht blockierende Hinweise / Lessons

- Die Datei `src-tauri/tauri.conf.json` und `src-tauri/capabilities/default.json` sind im gezeigten Stand nicht einsehbar. Für eine finale CRA-/Security-Prüfung sollte dort bestätigt werden, dass keine unnötigen Tauri-Berechtigungen (z. B. HTTP, Shell) aktiviert sind und die CSP keine Remote-Ressourcen erlaubt, die gegen AC-15 verstoßen würden. Das Produkt selbst darf unter seinen eigenen Restriktionen weiterhin lokal funktionieren; ein reines `csp`-Verbot von Remote-Ressourcen ist mit dem lokalen Charakter der App kompatibel.
- `search_notes` verwendet `LIKE ?1` mit vorformatiertem Pattern. Das ist SQL-Injection-sicher, erlaubt aber funktional die Wildcards `%` und `_` aus der Suchanfrage. Datenschutzrechtlich unbedenklich; für exakte Suchen wäre ein Escaping der LIKE-Wildcards eine optionale Verbesserung.
- Positive Feststellungen: AC-11 ist im sichtbaren UI-Code erfüllt, AC-12 ist erfüllt, AC-15 ist im sichtbaren Code erfüllt, AC-16 ist erfüllt.