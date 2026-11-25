Der interaktive Rebase verwandelt `git rebase -i` in einen visuellen Ablauf: die Commit-Warteschlange per Drag umordnen, Commits einzeln anwenden, Konflikte visuell lösen — keine Kommandozeile nötig.

## Start

Öffnen Sie das Branch-Fenster und tippen Sie in das Feld `Filter branches...`, um den Ziel-Branch zu finden, dann Rechtsklick darauf → `Rebase Current Branch onto '{branch}'`, um den aktuellen Branch auf das Ziel zu rebasen. Das Panel wechselt in die Rebase-Ansicht, die vier Phasen durchläuft:

![Branch-Suche im Branch-Fenster, dann Start des Rebase auf das Ziel](images/git-rebase-search.jpg)

## 1. Die Commit-Warteschlange bearbeiten

Die Commits Ihres Branches, die dem Ziel fehlen, bilden eine Warteschlange:

- **Ziehen** zum Umordnen;
- Jeder Commit hat ein Aktions-Dropdown: `pick` (behalten) / `squash` (in den vorherigen falten, Meldungen vereinen) / `fixup` (einfalten, Meldung verwerfen) / `edit` (zum Nachbessern anhalten) / `drop` (verwerfen);
- Gesquashte und gedroppte Commits klappen visuell zusammen.

![Die Rebase-Commit-Warteschlange — per Drag umordnen, Aktionen je Commit](images/git-rebase-queue.jpg)

## 2. Anwenden

`Start Rebase` spielt die Warteschlange Commit für Commit auf das Ziel ab, mit live Fortschrittsbalken; ohne Konflikte läuft es geradewegs durch. `Abort` steigt jederzeit aus.

## 3. Konflikte (falls welche)

Ein Konflikt beim Anwenden wechselt in die Konfliktansicht — eine Dateiliste plus dem Merge-Editor mit seinen vier Modi (`Merge` / `Split` / `Raw` / `AI`), wo jeder Konflikt-Hunk seine eigene Entscheidung erhält (`Ours` / `Theirs` / `Both` …; siehe Konflikte lösen). Sobald jede Datei gelöst ist, setzt `Continue` den Rebase fort.

![Die Konfliktansicht — Merge-Entscheidungen je Hunk](images/git-rebase-resolve-conflict.jpg)

## 4. Fertig

Beim Abschluss bestätigt das Panel `Branch '{branch}' was rebased onto '{target}'` und zeigt die Push-Karte — der Rebase hat die Historie umgeschrieben, der Remote-Branch muss zwangsaktualisiert werden:

- `Force Push with Lease` — empfohlen: überschreibt nur, wenn niemand sonst gepusht hat, und nutzt nie ein schlichtes `--force`;
- `Not now` — Push überspringen; der lokale Rebase ist bereits abgeschlossen.

![Die Fertig-Seite — Force Push with Lease oder Push überspringen](images/git-rebase-done.jpg)

> Die Sitzung bleibt während des Rebase nutzbar; das Schließen des Panels unterbricht keinen laufenden.
