Übersetzung verwandelt Text, dem Sie im Client begegnen, in die Sprache Ihrer Wahl: KI-Antworten im Chat, Commit-Meldungen, Code-Kommentare oder eine beliebige Auswahl im Code-Editor oder in einem Diff. Die Arbeit erledigt ein auf dem Server konfiguriertes Übersetzungsmodell — der Client sendet nur den Text, rendert das Ergebnis und cached es, damit Sie zwischen Original und Übersetzung hin- und herschalten können.

## Wo Übersetzung verfügbar ist

- **Copilot-Panel** — jede Nachrichten-Kopfzeile (eigene Nachrichten, Modell-Antworten und `Denkt nach...`-Blöcke) trägt ein Übersetzungssymbol. Ein Klick ersetzt den Nachrichtentext durch die Übersetzung; ein weiterer Klick stellt das Original wieder her;
- **Commit-Meldungsentwurf** — das Commit-Feld im Git-Panel zeigt eine Übersetzungstaste, sobald der Entwurf mehrere Zeilen umfasst. Es übersetzt den gesamten Entwurf an Ort und Stelle, und ein weiterer Klick holt Ihre ursprüngliche Formulierung zurück;
- **Commit-Detail-Popup** — das Hovern über einen Commit in der Git-Historie öffnet ein Detail-Popup; das Übersetzungssymbol neben dem Hash bringt die Commit-Meldung in Ihre Sprache;
- **Code-Editor & Diff-Ansicht** — wählen Sie beliebiges aus (Blockauswahlen zählen auch), klicken Sie mit der rechten Maustaste und wählen Sie `Übersetzen`. Ein kleines Globus-Badge markiert die übersetzten Zeilen in der Gutter, und eine Karte mit der Übersetzung öffnet sich neben der Auswahl.

## So funktioniert es

1. Der Client sendet den Text an den Server — bei Auswahlen aus Editor oder Diff zusätzlich mit dem Dateinamen, damit das Modell weiß, welche Kommentar-Syntax die Datei verwendet;
2. Der Übersetzungs-LLM des Servers überträgt ihn in die in der Web UI konfigurierte Zielsprache. Markdown-Formatierung (Überschriften, Listen, Tabellen, Code-Blöcke) bleibt erhalten; bei Datei-Auswahlen werden Kommentar-Marker wie `//` oder `#` als Dekoration erkannt und entfernt, während Code und Bezeichner unangetastet bleiben;
3. Der Client rendert das Ergebnis als Markdown dort, wo die Anfrage herkam — bei Chat-Nachrichten an Ort und Stelle, bei Editor- und Diff-Auswahlen in einer scrollbaren Karte.

## Was Sie brauchen

- Einen Server mit aktivierter Übersetzungsfunktion — die einmalige Einrichtung beschreibt *Server-Setup* (Abschnitt `LLM` der Web UI);
- Einen Desktop-Client, der mit diesem Server verbunden ist. Client-seitig ist nichts zu konfigurieren: Alle Oberflächen greifen automatisch auf dieselben Servereinstellungen zurück.

Ist die Übersetzung nicht konfiguriert, zeigt der Client statt eines Ergebnisses den Fehler des Servers an (zum Beispiel in der Übersetzungskarte).
