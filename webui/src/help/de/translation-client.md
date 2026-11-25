Sobald der Server konfiguriert ist, bietet jede Client-Oberfläche die Übersetzung über dasselbe Globus-Symbol an. Ergebnisse werden pro Eintrag gecacht, sodass das Hin- und Herschalten nie eine erneute Anfrage auslöst.

## Nachrichten im Copilot-Panel

Jede Nachrichten-Kopfzeile trägt ein Übersetzungssymbol — neben der Zeit bei eigenen Nachrichten, rechts neben Absendername und Zeit bei Modell-Antworten und rechts neben der `Denkt nach...`-Bezeichnung bei Reasoning-Blöcken:

- **Erster Klick** sendet das rohe Markdown der Nachricht an den Server und tauscht den Nachrichtentext gegen die Übersetzung. Während die Übersetzung angezeigt wird, leuchtet das Symbol auf;
- **Ein weiterer Klick** stellt den Originaltext wieder her;
- Die Übersetzung bleibt an der Nachricht haften, solange die Konversation offen ist — auch an `Denkt nach...`-Blöcken, sobald Sie sie ausklappen.

Übersetzte Nachrichten rendern normales Markdown, sodass Listen, Tabellen und Code-Blöcke ihre Form behalten.

![Eine Chat-Nachricht mit dem Übersetzungssymbol in der Kopfzeile und dem übersetzten Text darunter](images/translation-chat-message.jpg)

## Commit-Meldungsentwurf

Im Commit-Feld des Git-Panels erscheint neben der AI-Taste eine Übersetzungstaste, sobald der Entwurf mehr als eine Zeile umfasst:

- **Klicken** übersetzt den gesamten Entwurf — der übersetzte Text ersetzt den Entwurf in der Box als schreibgeschützte Vorschau;
- **Erneut klicken** bringt Ihre ursprüngliche Formulierung zurück (der Entwurf selbst wurde nie verändert);
- Eingaben sind deaktiviert, solange die Vorschau angezeigt wird; schalten Sie erst zurück und bearbeiten Sie dann weiter.

![Das Commit-Feld mit der Übersetzungstaste und der übersetzten Vorschau](images/translation-commit-draft.jpg)

## Commit-Detail-Popup

Hovern Sie in der Git-Historie über einen Commit, um das Detail-Popup zu öffnen. Neben der Hash-Kopiertaste sitzt ein Übersetzungssymbol: Klicken übersetzt die Commit-Meldung im Popup; ein weiterer Klick zeigt das Original. Die übersetzte Meldung wird wie das Original gerendert — Betreffzeile, Markdown-Text und Code-Blöcke inklusive.

![Das Commit-Detail-Popup — Kopieren- und Übersetzen-Symbole in der Hash-Zeile](images/translation-git-tip.jpg)

## Auswahlen im Code-Editor

Wählen Sie im Code-Editor Text aus — normale und Block-(rechteckige) Auswahlen funktionieren —, klicken Sie mit der rechten Maustaste und wählen Sie `Übersetzen` (der erste Menüeintrag):

1. Über der Zeilennummer der ersten ausgewählten Zeile erscheint ein gefülltes Globus-Badge;
2. Eine Übersetzungskarte öffnet sich sofort mit einem *Translating…*-Platzhalter und füllt sich, wenn der Server antwortet;
3. Die Karte ist klebrig: Sie bleibt beim Scrollen an Ort und Stelle, und nur ihr Schließen-Button, `Esc` oder ein Klick außerhalb verwerfen sie. Das Hovern über das Badge holt sie zurück.

Jede übersetzte Auswahl erhält ihr eigenes Badge, und das erneute Übersetzen derselben Zeilen zeigt einfach die gecachte Karte wieder an. Beim Öffnen einer anderen Datei verschwinden die Badges.

![Eine Auswahl im Code-Editor mit Übersetzen im Kontextmenü](images/translation-editor-menu.jpg)

## Auswahlen in der Diff-Ansicht

Die Diff-Ansicht funktioniert genauso: Zeilen im Diff-Text auswählen, rechte Maustaste, `Übersetzen`. Das Badge ersetzt die neue Zeilennummer der ersten ausgewählten Zeile, und die Ergebniskarte dockt darunter an. Auswahlen im Bearbeitungsmodus lassen sich ebenfalls übersetzen; der Wechsel zu einer anderen Datei oder einem anderen Commit leert die Badges.

![Ein Diff mit übersetzter Auswahl — Gutter-Badge und Übersetzungskarte](images/translation-diff-card.jpg)

## Was übersetzt wird

Der Server teilt dem Modell mit, woher der Text stammt:

- **Prosa** (Chat-Nachrichten, Commit-Meldungen) — die Markdown-Formatierung bleibt erhalten, nur die natürliche Sprache wird übersetzt;
- **Datei-Auswahlen** (Editor, Diff) — der Dateiname reist mit der Anfrage, sodass das Modell die Kommentar-Marker der Datei kennt (`//`, `#`, `--`, …). Kommentar-Marker werden entfernt und der Kommentartext als Prosa übersetzt; Markdown innerhalb des Kommentars (Überschriften, Listen, Code-Blöcke) überlebt. Ist die Auswahl gewöhnlicher Code, werden nur Kommentare und String-Literale übersetzt — Bezeichner und Struktur bleiben unberührt.
