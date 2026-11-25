De server heeft een ingebouwde Model Context Protocol-host, en de sectie `MCP` van de Web UI configureert die: Code Agents (Claude Code, Codex, Kimi Code, Pi, OpenCode, Qwen Code, DeepSeek Harness, …) kunnen vanuit hun eigen chat verbinden en direct op je machine werken — terminals lezen en besturen en zelfs EasyEDA Pro bedienen.

## Serverinstellingen

Schakel de MCP-server in en stel in:

- `Port` (standaard `6180`) en `Bind Address` — `127.0.0.1` houdt het endpoint lokaal; `0.0.0.0` opent het voor het netwerk;
- `Access Token` — vereist voor niet-localhost-verzoeken: externe bellers moeten hem als Bearer-token meesturen, terwijl verzoeken van localhost altijd zijn toegestaan. Hij is nooit leeg — een verse installatie levert een willekeurige token; de dobbelsteenknop genereert een nieuwe, de kopieerknop deelt hem uit;
- `Stream Throttle` — het minimale interval tussen schermstream-pushes naar verbonden agents.

Wijzigingen gelden na `Save`; de server herlaadt zonder herstart.

## Endpoints

Elke mogelijkheidenset wordt op eigen URL `http://localhost:{port}/{id}/mcp` aangeboden:

- **Terminal MCP** (`/terminal/mcp`) — het ingebouwde terminaloppervlak. Agents kunnen terminals opsommen, het zichtbare scherm en scrollback lezen, ruwe invoer en benoemde toetsen typen, het formaat wijzigen en op een prompt wachten. Tool-aanroepen richten zich op één terminal via zijn Terminal-ID — rechtsklik op een terminal → `MCP` → `Copy Terminal ID` en plak de ID in de agentcontext (zie Terminal MCP in de sectie Terminals);
- **EasyEDA Pro MCP** (`/easyeda/mcp`) — een ingebouwde MCP-addon die EasyEDA Pro naar agents overbrugt: een klein vast gereedschapsset plus een doorzoekbare catalogus van `eda.*`-API-aanroepen, doorgestuurd naar de EasyEDA-connectorextensie via WebSocket. Schakel de addon eerst in de sectie `Addons` in — het endpoint verschijnt in de lijst zodra geïnstalleerd, en de badge toont of de addondaemon verbonden is. Het `Timeout`-veld van de addon begrenst elk overbrugde verzoek (verhoog voor trage EDA-bewerkingen);
- **Andere MCP-addons** — elke geïnstalleerde addon van het type MCP voegt zijn endpoint op dezelfde manier toe, elk met een live badge verbonden/verbroken.

Kies je agent-CLI in het uitklapmenu boven de lijst; de kopieerknop naast een endpoint geeft dan de direct uitvoerbare instelopdracht voor die app.

## Een agent verbinden

De token hoef je nooit handmatig te plakken. De server onderhoudt een automatisch gegenereerd omgevingsprofiel `Vibe Terminal MCP Bearer Token` (zie de sectie Environment Variables) dat `VTERM_MCP_TOKEN` exporteert; pas het toe op de terminal voordat je een agent verbindt en gebruik daarna de gekopieerde opdracht:

- **Shell-opdracht-apps** (Claude Code, Codex) — voer de gekopieerde `mcp add`-opdracht uit in een gewone terminal. Codex leest de token uit `VTERM_MCP_TOKEN` via `--bearer-token-env-var`, dus geen geheim in de opdrachtregel;
- **Conversatie-apps** (Kimi Code, Pi, OpenCode, Qwen Code, DeepSeek Harness) — start de app met het profiel toegepast en stuur de gekopieerde prompt in de conversatie. De agent werkt zijn MCP-configuratie rechtstreeks bij. Kimi gebruikt `bearerTokenEnvVar`; Pi, OpenCode en Qwen Code gebruiken het Bearer-token uit `VTERM_MCP_TOKEN`, zodat het token zelf nooit in de chat verschijnt.

Binnen Pi en Qwen Code inspecteert `/mcp` verbindingen.

De kaart `Command Templates` onder de instellingen bevat de instelopdrachten per app: bewerk een item, overschrijf een ingebouwde of voeg sjablonen toe voor nieuwe agent-CLI's — het voorbeeld toont precies wat de kopieerknop oplevert.
