Wenn ein Rebase oder Merge auf Konflikte stößt, wechselt das Panel in die Konfliktansicht: links die Konflikt-Dateiliste, rechts der Merge-Editor. Konfliktdateien erscheinen auch in der Liste `CHANGES` mit einem `!`-Abzeichen — ein Klick darauf springt direkt in diese Ansicht statt in ein schlichtes Diff. Der Editor bietet vier Lösungsmodi, zwischen denen Sie jederzeit wechseln können.

![Die Konfliktansicht — links Dateiliste, rechts Merge-Editor](images/git-conflicts-view.jpg)

## Die Konflikt-Dateiliste

Dateien werden nach Zustand gruppiert: die automatisch erfolgreich gemergten (automatisch gestagt) und die noch manueller Aufmerksamkeit bedürfenden. Klicken Sie eine Datei an, um sie im Editor zu öffnen; die Liste aktualisiert sich live während Sie lösen, bis sie leer ist.

## Die vier Lösungsmodi

- **Merge** (Standard) — Konflikte werden Hunk für Hunk präsentiert; pro Hunk `take ours` / `take theirs` wählen oder das gemergte Ergebnis inline bearbeiten;
- **Split** — ein Diff wird in feinere Zeilen-Segmente zerlegt, sodass beide Seiten stückweise innerhalb eines Hunks übernommen werden können;
- **Raw** — schlichtes Bearbeiten mit `diff3`-Konfliktmarkern (`<<<<<<<` / `=======` / `>>>>>>>`) zum Handjustieren;
- **AI** — ein Agent löst die Datei Hunk für Hunk: das LLM analysiert jeden Konflikt-Hunk und erzeugt Lösungskarten mit Erklärung und Konfidenz. Sie können Anweisungen tippen, um die Strategie zu steuern (z. B. „vorzugsweise theirs, die Log-Anweisungen behalten“), pausieren, in einem anderen Modus feinjustieren und fortfahren — bereits gelöste Hunks werden übersprungen.

In jedem Modus wird die Lösung in die Datei Ihrer Sitzung geschrieben und gestagt, exakt wie eine manuelle Bearbeitung.

## Den Rebase fortsetzen

Sobald jeder Konflikt gelöst ist, geht der Rebase zum nächsten Commit weiter; Sie können auch jederzeit abbrechen und in den Vor-Rebase-Zustand zurückkehren.
