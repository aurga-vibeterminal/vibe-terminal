Der Bereich `CHANGES` des Git-Panels deckt die alltägliche Commit-Schleife ab: Änderungen prüfen, stagen, verwerfen, stashen und mit Meldung committen.

## Stagen und Entstagen

- Unstaged Änderungen stehen unter `CHANGES`, jede Datei mit einem Status-Abzeichen — `M` geändert, `A` hinzugefügt, `D` gelöscht, `?` unbeobachtet, `!` Konflikt (Submodul-Zeilen tragen ein zusätzliches `SUB`-Tag);
- Dauert der Zeiger über einer Zeile an, erscheinen Inline-Aktionsbuttons: `+` stagt eine unstaged Datei, `−` entstagt eine gestagte, und `×` auf einer unstaged Zeile verwirft sie; die Buttons `Stage All Changes` / `Unstage All Changes` in den Abschnittsköpfen erledigen alles auf einmal;
- `Ctrl`-Klick schaltet Dateien in eine Mehrfachauswahl und `Shift`-Klick wählt einen Bereich — ein Aktionsbutton gilt dann jeder gewählten Datei;
- Ein Klick auf einen Dateinamen öffnet ihr Diff, sodass Sie die Änderung vor der Entscheidung prüfen können;
- Der Rechtsklick auf eine Datei bietet `View File History`, `Stash This File`, `Copy Full Path` / `Copy Relative Path`, `Reveal in File Explorer` und eine `cd`-Abkürzung, die das Terminal in den Ordner der Datei springen lässt.

![Der Abschnitt STAGED CHANGES — gestagte Dateien mit Entstage-Buttons pro Zeile](images/git-staging-staged.jpg)

## Committen

1. Stagen, was hinein soll;
2. Eine Meldung in die Commit-Box tippen (`Ctrl+Enter` committet) oder den KI-Button für eine Generierung klicken (siehe LLM-Unterstützung);
3. `Commit` klicken.

![Die Commit-Box — Meldungseingabe mit dem KI-Generieren-Button](images/git-staging-commit.jpg)

Nach dem Committen den Verlaufskopf bewegen — die Leiste mit dem Branchnamen über dem Commit-Verlauf — und `Fetch`-, `Pull`- und `Push`-Buttons blenden rechts ein. `Push` sendet den aktuellen Branch sofort an sein Upstream; für Optionen wie `Force with lease` nutzen Sie stattdessen den Push-Dialog der Branchliste (siehe Push, Checkout & Mehr).

![Dauern auf dem Verlaufskopf an, erscheinen die Fetch-, Pull- und Push-Buttons](images/git-staging-push.jpg)

## Stashen

- Der Archiv-Button im Kopf von `CHANGES` öffnet den Dialog `Stash All Changes`, in dem eine optionale Meldung den Stash beschreibt;
- Der Rechtsklick auf gewählte unstaged Dateien bietet `Stash This File` (oder `Stash <n> Selected Files` bei Mehrfachauswahl), um genau diese Pfade zu stashen — Submodul-Zeilen sind ausgeschlossen;
- Stashes sammeln sich unter `STASHES` als Einträge `WIP on <branch>`; Aufklappen listet die erfassten Dateien, ein Klick auf eine Datei zeigt ihr Diff;
- Über einer Stash-Zeile erscheinen `Apply` (wiederherstellen und behalten), `Pop` (anwenden und entfernen) und `Drop` (entfernen ohne Anwenden) — `Pop` und `Drop` fragen zuerst nach Bestätigung, und das Rechtsklick-Menü ergänzt `View Diff`;
- Der Abschnitt `STASHES` bleibt verborgen, solange die Stash-Liste leer ist.

![Der Dialog Stash All Changes mit seiner optionalen Meldung](images/git-staging-stash-all.jpg)

![Der Abschnitt STASHES — ein Stash zu seinen Dateien aufgeklappt, mit Apply-, Pop- und Drop-Buttons](images/git-staging-stashes.jpg)

![Pop wendet den Stash an und entfernt ihn nach einer Bestätigung aus der Liste](images/git-staging-stash-pop.jpg)

## Verwerfen

`Discard Changes` auf einer unstaged Datei verwirft alle uncommitteten Änderungen in ihr — es gibt einen Bestätigungsdialog und die Aktion **kann nicht rückgängig gemacht werden**.

> Ein Commit umfasst nur gestagten Inhalt; unstaged Änderungen bleiben im Arbeitsbaum.
