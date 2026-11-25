Drücken Sie im Desktop-Client überall zweimal schnell `Ctrl` (innerhalb ~300 ms), um den Spracheingabe-Dialog zu öffnen.

## Aufnahme

Der Dialog startet sofort die Aufnahme: die flüssige Wellenform folgt Ihrer Stimme, die Auswahl oben wechselt Mikrofone (die Wahl wird gemerkt), und ein Timer zeigt die vergangene Zeit. `Stop` beendet die Aufnahme und sendet sie zur Transkription (`Reset` startet neu).

![Die Aufnahmephase — Wellenform, Mikronauswahl und Timer](images/voice-client-record.jpg)

## Verarbeitung

Während der Server das Audio transkribiert, läuft eine Partikelstrom-Animation.

## Bearbeiten

Das Transkript landet in einer editierbaren Textbox:

- Der Modus-Pill schaltet zwischen `original` und `polished`; `Polish` lässt das LLM einmal laufen, und `Auto polish` tut es direkt nach jeder Transkription;
- `Re-record` verwirft den Text und beginnt von vorn; `Insert` nimmt erneut auf und fügt das neue Transkript am Caret-Zeichen in den bestehenden Text ein (eine Auswahl ersetzend), sodass Sie bereits Diktiertes erweitern oder berichtigen können — der Cursor landet direkt hinter dem eingefügten Schnipsel;
- Fahren Sie über die Textbox, um oben rechts einen Kopieren-Button aufzudecken: ein Klick kopiert den Text und er klappt zu einem Häkchen; Bearbeiten des Textes oder Moduswechsel rüsten ihn für ein weiteres Kopieren;
- `Submit` (oder `Ctrl+Enter`) tippt den Text ins aktive Terminal; `Esc` schließt den Dialog.

![Die Bearbeitungsphase — Modus-Pill, Auto Polish, die Aktionszeile mit Insert und der Hover-Kopieren-Button](images/voice-client-editing.jpg)

## Polieren einrichten

`Polish` braucht ein Voice-Polish-LLM: aktivieren Sie im Bereich `LLM` der Web UI in der Funktionsliste `Enable voice text polish` (ein regulärer Provider plus dieser Schalter genügen). Ohne ihn bleibt der `Polish`-Button ausgegraut und die Auto-Polish-Option verborgen.
