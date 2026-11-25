Wenn Sie einen Code-Agenten (Claude Code, Codex, Kimi, Pi, OpenCode …) in einem Terminal laufen lassen, ist sein Gespräch in diesem Terminalfenster gefangen — weggeblättert zwischen Befehlsausgaben, fort, sobald der Bildschirm weiterläuft. Der `Code-Agent-Gesprächsverlauf` hebt das ganze Gespräch in ein eigenes Panel: jeder Austausch strukturiert dargestellt, frühere Gespräche umschaltbar, Meldungen übersetzbar — und Sie können aus dem Panel heraus Befehle direkt ins Terminal senden.

## Das Panel öffnen

Klicken Sie in der Titelleiste rechts auf den Chat-Button (`AI`), um das Copilot-Panel ein-/auszublenden. Das Dropdown oben listet **alle früheren Gespräche** des in der aktiven Terminalsitzung laufenden Agents, neueste zuerst; eines auswählen lädt es.

![Das Copilot-Panel — das Gesprächs-Dropdown und der Nachrichtenfluss](images/terminal-agent-history.jpg)

## Wie es funktioniert

- Der Server weiß, welche Agent-App im Terminal läuft, und analysiert ihre **nativen Sitzungsdateien** (das JSONL-/Verlaufsformat jedes Agents unter seinem Projektverzeichnis); Änderungen an der Datei streamen live herein und halten die Gesprächsansicht aktuell;
- Unterstützte Agents: `claude` / `codex` / `kimi` / `pi` / `opencode`; andere Apps haben noch kein Gesprächs-Panel;
- Meldungen tragen eine Rolle (Sie / Agent) und einen Typ (Text / **Thinking**-Reasoning), dazu Modellname und Zeit, gerendert als Markdown.

## Panel-Funktionen

- **Gespräche wechseln**: das Dropdown `Conversations` oben bewegt zwischen früheren Sitzungen; jeder Eintrag zeigt eine Kurzbeschreibung und seine Zeit;
- **Übersetzen**: jede Agent-Meldung lässt sich auf Wunsch übersetzen; die Übersetzung ersetzt den Text an Ort und Stelle (und lässt sich zurückumschalten);
- **Befehle direkt senden**: tippen Sie unten in die Box (Shift+Enter für einen Umbruch) und drücken Sie Enter — der Befehl geht direkt ins aktuelle Terminal, wie das Tippen in die eigene Eingabebox des Agents, ohne die Terminal-Ansicht zu belegen.

## Zusammenspiel mit anderen Funktionen

- Panel und Terminal sind zwei Ansichten derselben Sitzung: Die Fragen des Agents im Terminal werden weiterhin dort (oder über Auto Confirm) beantwortet; das Panel dient dem Durchsehen des Verlaufs, nicht dem Abfangen der Interaktion;
- Kombiniert mit `Terminal MCP` (vorige Seite) können externe Agents auch den Terminal-Bildschirm lesen und Befehle ausführen — das hier gezeigte Gespräch plus die MCP-Aktionen bilden die vollständige Aufzeichnung der Agent-Arbeit.
