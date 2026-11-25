Code-Agents (Claude Code, Codex, Kimi, Pi, OpenCode …) wissen nur, was ihren Input erreicht. `Kontext hinzufügen` ist der Weg, ihnen Dateipfade und Code-Schnipsel ohne erneutes Tippen zu übergeben: Ziehen Sie Dateien aus dem `Explorer`, oder ziehen Sie ausgewählten Text aus dem Code-Editor und der Git-Diff-Ansicht direkt ins Terminal des Agents oder in die Copilot-Chat-Eingabe.

## Dateien hineinziehen

Drücken Sie eine Datei — oder eine Mehrfachauswahl — in der Dateiliste des `Explorer` und ziehen Sie sie:

- **Auf ein Terminal**: die vollständigen Pfade werden am Ablegepunkt eingetippt, leerzeichengetrennt, bereit, Teil Ihres nächsten Prompts an den Agenten zu werden;
- **Auf die Copilot-Chat-Eingabe**: die Pfade werden in den Nachrichtenentwurf eingefügt, einer pro Zeile, sodass Sie einen Mehrdateien-Prompt vor dem Senden zusammenstellen können.

Pfade, die aus dem Dateimanager Ihres Betriebssystems gezogen und auf einem Terminal abgelegt werden, werden automatisch maskiert, sodass Namen mit Leerzeichen intakt überleben.

![Eine Datei aus der Explorer-Dateiliste in ein Terminal ziehen tippt ihren vollständigen Pfad ein](images/terminal-context-drag-file.gif)

## Ausgewählten Code als Kontext ziehen

Wählen Sie Text im Code-Editor oder in einem Git-Diff und ziehen Sie die Auswahl selbst auf ein Terminal:

- Läuft im Terminal ein **Code-Agent**, fügt das Ablegen einen *Kontextblock* ein — den Dateipfad, den Zeilenbereich und den in einen Code-Fence gehüllten Schnipsel:

````text
File: crates/vterm/src/ui/terminal_view.rs
Lines: 558-566
```
fn can_accept_native_drop(&self, info: &DndInfo) -> bool { … }
```
````

- Steht das Terminal an einem **schlichten Shell-Prompt**, tippt das Ablegen stattdessen den rohen ausgewählten Text ein, sodass dieselbe Geste auch für gewöhnliche Befehle funktioniert;
- Wird der Diff zu einem bestimmten Commit betrachtet, trägt der Block zusätzlich eine `Commit:`-Zeile, die die exakte Revision festnagelt.

![Eine Editor-Auswahl in ein Agent-Terminal ziehen legt einen File/Lines-Kontextblock ab](images/terminal-context-drag-selection.mp4)

## Kontext kopieren

Derselbe Block lebt im Rechtsklick-Menü: `Copy Context` im Code-Editor und in der Diff-Ansicht kopiert ihn in die Zwischenablage. Fügen Sie ihn in die Copilot-Chat-Eingabe ein — oder in jeden anderen Agenten —, wenn Ziehen unpraktisch ist. Die Animation oben zeigt den entstandenen Block in der Eingabe des Agents.

## Tipps

- Mehrfachauswahl im `Explorer`, um mehrere Pfade auf einmal abzulegen;
- Die Copilot-Chat-Eingabe sendet wie getippte Eingabe direkt ins Terminal (siehe `Code-Agent-Verlauf`), sodass dort hinzugefügter Kontext den Agenten exakt so erreicht, als hätten Sie ihn getippt;
- Kombinieren Sie mit `Auto Confirm`, damit der Agent ohne weitere Nachfragen auf dem abgelegten Kontext agiert.
