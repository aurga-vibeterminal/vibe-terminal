The server ships with a built-in Model Context Protocol host, and the `MCP` section of the Web UI configures it: Code Agents (Claude Code, Codex, Kimi Code, Pi, OpenCode, Qwen Code, DeepSeek Harness, …) can connect from their own chat and work directly on your machine — reading and driving terminals, and even operating EasyEDA Pro.

## Server settings

Enable the MCP server and set:

- `Port` (default `6180`) and `Bind Address` — `127.0.0.1` keeps the endpoint localhost-only; `0.0.0.0` opens it to the network;
- `Access Token` — required for non-localhost requests: remote callers must present it as a Bearer token, while requests from localhost are always allowed without one. It is never empty — a fresh install ships a random token; the dice button regenerates it and the copy button hands it out;
- `Stream Throttle` — the minimum interval between screen-stream pushes to connected agents.

Changes apply on `Save`; the server hot-reloads without a restart.

## Endpoints

Every capability set is served at its own URL, `http://localhost:{port}/{id}/mcp`:

- **Terminal MCP** (`/terminal/mcp`) — the built-in terminal surface. Agents can list terminals, read the visible screen and scrollback, type raw input and named keys, resize a terminal, and wait for a prompt. Tool calls target one terminal by its Terminal ID — right-click a terminal → `MCP` → `Copy Terminal ID` and paste the ID into the agent's context (see Terminal MCP in the Terminals section);
- **EasyEDA Pro MCP** (`/easyeda/mcp`) — a built-in MCP addon that bridges EasyEDA Pro to agents: a small fixed tool set plus a searchable catalog of `eda.*` API calls, forwarded to the EasyEDA connector extension over WebSocket. Enable the addon in the `Addons` section first — its endpoint appears in the list once installed, and the badge shows whether the addon daemon is currently connected. The addon's `Timeout` field caps each bridged request (raise it for slow EDA operations);
- **Other MCP addons** — any installed MCP-type addon adds its endpoint here the same way, each with a live connected/disconnected badge.

Pick your agent CLI in the dropdown above the list; the copy button next to an endpoint then hands you the ready-to-run setup command for that app.

## Connecting an agent

The token never needs to be pasted by hand. The server maintains a generated `Vibe Terminal MCP Bearer Token` environment profile (see the Environment Variables section) that exports `VTERM_MCP_TOKEN`; apply it to the terminal before connecting an agent, then use the copied command:

- **Shell-command apps** (Claude Code, Codex) — run the copied `mcp add` command in a plain terminal. Codex reads the token from `VTERM_MCP_TOKEN` via `--bearer-token-env-var`, so nothing secret ends up in the command line;
- **Conversation apps** (Kimi Code, Pi, OpenCode, Qwen Code, DeepSeek Harness) — start the app with the profile applied, then send the copied prompt inside its conversation. The agent updates its MCP configuration directly. Kimi uses `bearerTokenEnvVar`; Pi, OpenCode, and Qwen Code use the Bearer token from `VTERM_MCP_TOKEN`, so the token itself never appears in the chat.

Inside Pi and Qwen Code, `/mcp` inspects connections.

The `Command Templates` card below the settings holds the per-app setup commands: edit an entry, override a built-in, or add templates for new agent CLIs — the preview shows exactly what the copy button will produce.
