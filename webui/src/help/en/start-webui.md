The Web UI is the server's own configuration page: every server feature is organised into a section, listed in the left sidebar. It is not a terminal — day-to-day work happens in the desktop client or `vtermc` (see Overview). This page explains the layout and common behaviour; the feature chapters cover each section's settings in detail.

## Layout

- **Sidebar** — the section list (below), the help icon beside the server name, and the app version at the bottom; on narrow screens it slides in from the left and the header's hamburger button toggles it;
- **Header** — the current section's title plus, at the top-right, the language selector and the light/dark toggle (on narrow screens the help icon sits there too);
- **Content** — the selected section's form, centred and scrollable.

## Sections

In sidebar order:

- `Server` — port, TLS certificate and authentication settings: the `Secret Key`, the pairing `QR Code`, the relay address and the auto-start toggle all live here (see Connecting a Client and Setup Guide). On Linux the auto-start hint also explains display-manager automatic login for reaching the desktop before a local login (see Server Setup in Remote Desktop & Simulator);
- `Terminal` — terminal analysis: prompt detection, auto-confirmation and LLM classification (see Auto Confirm);
- `UART` — serial terminal profiles backed by local UART ports (see UART Serial);
- `Addons` — installed addons: enable, edit or upload them (LLM providers are addons — see LLM);
- `LLM` — assigns an LLM provider to each AI feature (see LLM);
- `Voice` — speech recognition provider, model and transcription settings (see Server Setup in Voice Input);
- `Environment` — environment-variable collections in a tree of groups (see Environment Variables);
- `MCP` — the terminal MCP server exposed to code agents (see Terminal MCP);
- `DDNS` — DDNS provider and sync settings (see DDNS);
- `Port Forwarding` — the forwarded-port pool and the live TLS forwards (see Port Forwarding);
- `Static HTTP` — folders hosted over HTTP and the active file shares (see Host with HTTP Server);
- `Remote Desktop` — capture, input and stream settings for remote desktop (see Server Setup in Remote Desktop & Simulator);
- `LLM Stats` — token usage, request counts and performance metrics across all LLM features.

## Saving & tips

- Most sections save automatically shortly after an edit — a saving indicator and any error appear at the bottom of the section;
- `MCP`, `DDNS` and `Remote Desktop` hold edits until `Save` is pressed: while edits are pending an `Unsaved changes` hint appears beside the buttons, and `Reset` discards them;
- Opened from the mobile client, the Web UI follows the app's language and theme and hides its own language and theme controls.
