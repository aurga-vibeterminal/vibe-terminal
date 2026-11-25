Der Server bringt einen eingebauten Model-Context-Protocol-Host mit; der `MCP`-Bereich der Web UI konfiguriert ihn: Code Agents (Claude Code, Codex, Kimi Code, Pi, OpenCode, Qwen Code, DeepSeek Harness, …) können aus ihrem eigenen Chat heraus eine Verbindung aufbauen und direkt auf Ihrer Maschine arbeiten — Terminals lesen und steuern und sogar EasyEDA Pro bedienen.

## Servereinstellungen

Aktivieren Sie den MCP-Server und stellen Sie ein:

- `Port` (Standard `6180`) und `Bind Address` — `127.0.0.1` hält den Endpunkt lokal; `0.0.0.0` öffnet ihn im Netzwerk;
- `Access Token` — für Nicht-localhost-Anfragen erforderlich: entfernte Aufrufer müssen ihn als Bearer-Token mitgeben, Anfragen vom localhost sind immer erlaubt. Er ist nie leer — eine Neuinstallation liefert ein zufälliges Token; der Würfel-Knopf erzeugt ein neues, der Kopieren-Knopf gibt es heraus;
- `Stream Throttle` — das minimale Intervall zwischen Bildschirm-Stream-Pushes an verbundene Agents.

Änderungen greifen mit `Save`; der Server lädt ohne Neustart neu.

## Endpunkte

Jede Fähigkeitsmenge wird unter ihrer eigenen URL `http://localhost:{port}/{id}/mcp` ausgeliefert:

- **Terminal MCP** (`/terminal/mcp`) — die eingebaute Terminal-Oberfläche. Agents können Terminals auflisten, sichtbaren Bildschirm und Scrollback lesen, Rohtext und Tastennamen eingeben, die Größe ändern und auf einen Prompt warten. Tool-Aufrufe adressieren ein Terminal über seine Terminal-ID — Rechtsklick auf ein Terminal → `MCP` → `Copy Terminal ID` und die ID in den Agent-Kontext einfügen (siehe Terminal MCP im Terminals-Abschnitt);
- **EasyEDA Pro MCP** (`/easyeda/mcp`) — ein eingebautes MCP-Addon, das EasyEDA Pro an Agents brückt: ein kleiner fester Werkzeugsatz plus ein durchsuchbarer Katalog von `eda.*`-API-Aufrufen, weitergeleitet an die EasyEDA-Connector-Erweiterung über WebSocket. Aktivieren Sie das Addon zuerst im `Addons`-Bereich — sein Endpunkt erscheint nach der Installation in der Liste, und das Badge zeigt, ob der Addon-Daemon gerade verbunden ist. Das `Timeout`-Feld des Addons begrenzt jede gebrückte Anfrage (für langsame EDA-Operationen erhöhen);
- **Weitere MCP-Addons** — jedes installierte MCP-typisierte Addon fügt seinen Endpunkt auf dieselbe Weise hinzu, jeweils mit Live-Badge verbunden/getrennt.

Wählen Sie Ihre Agent-CLI im Dropdown über der Liste; der Kopieren-Knopf neben einem Endpunkt liefert dann den lauffähigen Setup-Befehl für diese App.

## Einen Agent verbinden

Das Token muss nie von Hand eingefügt werden. Der Server pflegt ein generiertes `Vibe Terminal MCP Bearer Token`-Umgebungsprofil (siehe Abschnitt Environment Variables), das `VTERM_MCP_TOKEN` exportiert; wenden Sie es vor dem Verbinden auf das Terminal an und nutzen Sie dann den kopierten Befehl:

- **Shell-Befehls-Apps** (Claude Code, Codex) — führen Sie den kopierten `mcp add`-Befehl in einem normalen Terminal aus. Codex liest das Token über `--bearer-token-env-var` aus `VTERM_MCP_TOKEN`, sodass kein Geheimnis in der Kommandozeile landet;
- **Dialog-Apps** (Kimi Code, Pi, OpenCode, Qwen Code, DeepSeek Harness) — starten Sie die App mit angewandtem Profil und senden Sie die kopierte Aufforderung in ihrer Konversation. Der Agent aktualisiert seine MCP-Konfiguration direkt. Kimi verwendet `bearerTokenEnvVar`; Pi, OpenCode und Qwen Code verwenden das Bearer-Token aus `VTERM_MCP_TOKEN`, sodass das Token selbst nie im Chat erscheint.

In Pi und Qwen Code inspiziert `/mcp` Verbindungen.

Die `Command Templates`-Karte unter den Einstellungen hält die Setup-Befehle pro App: Einträge bearbeiten, eingebaute überschreiben oder Vorlagen für neue Agent-CLIs hinzufügen — die Vorschau zeigt exakt, was der Kopieren-Knopf liefern wird.
