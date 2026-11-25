Jedes Terminal ist ein vollwertiger Bürger des MCP (Model Context Protocol): Externe Agents steuern Terminals über die MCP-Werkzeuge des Servers — Terminals auflisten, den Bildschirm lesen, Befehle ausführen, auf Abschluss warten — sodass der Coding-Agent, den Sie ohnehin nutzen, echte interaktive Shells bedient.

## Die Terminal-MCP-Werkzeuge des Servers

Um Terminal-Sitzungen kann ein MCP-Client:

- `list_terminals` — alle offenen Terminals; `get_terminal` — Titel, cwd und App-Status eines Terminals;
- `get_screen` — den aktuellen sichtaren Bildschirmtext; `get_scrollback` — kürzliche Verlaufszeilen;
- `send_command` — eine Befehlszeile ausführen (Enter angehängt); `send_input` — Rohtext ohne Enter (`y` bestätigen, Formulare füllen); `send_key` — benannte Tasten (Enter, Ctrl+C, Pfeile);
- `clear_terminal` — den Puffer leeren; `resize_terminal` — das PTY skalieren; `wait_terminal` — blockieren, bis das Terminal nicht mehr beschäftigt ist.

## Ein Terminal adressieren: die Terminal-ID

Werkzeugaufrufe zielen über die Terminal-ID auf ein Terminal. Rechtsklick im Terminal → `MCP` → `Copy Terminal ID`, dann die ID Ihrem Agenten einfügen, damit er genau dieses Terminal steuert.

![Terminal-Rechtsklick — das MCP-Untermenü](images/terminal-mcp-menu.jpg)

## Agent-App-Anmeldung mit einem Klick

Das `MCP`-Untermenü listet auch die auf dem Server erkannten Agent-Apps (Editor-CLIs und Ähnliches). Ein Klick auf eine App tippt einen fertigen `mcp add`-Befehl ins Terminal — **ohne abschließenden Zeilenumbruch**; prüfen Sie ihn und drücken Sie selbst Enter, um den MCP-Dienst dieses Servers bei diesem Agenten zu registrieren. Einträge bleiben verborgen, während eine App bereits in der Sitzung läuft, damit Text nicht in der UI der App landet.

> Typischer Ablauf: die Terminal-ID zum Agenten kopieren → `send_command` startet einen Build → `wait_terminal` bis fertig → `get_screen` liest das Ergebnis. Zusammen mit Auto Confirm (siehe das Terminal-Kapitel) laufen lange interaktive Aufgaben nahezu unbeaufsichtigt.
