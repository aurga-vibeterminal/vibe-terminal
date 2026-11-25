Add-ons erweitern den Server um neue Fähigkeiten, ohne den Kern anzufassen: LLM-Provider, Spracherkennungs-Backends, Dynamic-DNS-Updater, zusätzliche MCP-Werkzeugflächen und die Erkennung von Code-Agent-Apps leben alle als Add-ons. Verwaltet werden sie im `Addons`-Bereich der Web UI.

## Kategorien

Jedes Add-on deklariert einen Typ, und der Bereich gruppiert die Karten entsprechend:

- **Applications** — bringen dem Terminal-Observer eine Code-Agent-CLI bei (Claude Code, Codex, Kimi Code, Pi und OpenCode sind eingebaut): Prozesserkennung, Beschäftigt/Leerlauf-Status, Automatisierungs-Hooks;
- **LLM Providers** — Chat-Completion-Backends, aus denen die KI-Funktionen wählen (ein OpenAI-kompatibler Provider ist eingebaut);
- **Voice / ASR** — Spracherkennungs-Backends (das eingebaute `vibe-voice`-Add-on zeigt auf den beaufsichtigten lokalen ASR-Server);
- **Dynamic DNS** — halten einen Hostnamen auf der aktuellen Adresse (Cloudflare ist eingebaut);
- **MCP** — zusätzliche MCP-Werkzeugflächen auf dem MCP-Host (EasyEDA Pro ist eingebaut);
- alles andere landet in der Gruppe **Other**.

## Eingebaute und hochgeladene Add-ons

- **Eingebaute** Add-ons werden mit dem Programm ausgeliefert und sind immer verfügbar. Manifest und Quelltext bleiben unberührt — ein überschriebener Quelltext lässt sich jederzeit wiederherstellen;
- **Hochgeladene** Add-ons sind Ihre eigenen: ein Zip mit Manifest und JavaScript-Quelltext, installiert über die `Upload Addon`-Schaltfläche;
- Das `Modified`-Badge kennzeichnet ein Add-on, dessen Quelltext Sie überschrieben haben.

Jede Karte zeigt Typ, Badges und einen Aktivierungs-Schalter; ausgeklappt erscheinen Beschreibung, Autor, Konfigurationsfelder und Quelltext.

![Der Addons-Bereich — gruppierte Karten mit Badges und Aktivierungs-Schaltern](images/addon-section.jpg)

## Konfiguration

Das Manifest eines Add-ons deklariert seine Konfigurationsfelder, und die ausgeklappte Karte macht daraus ein Formular. Werte werden pro Add-on gespeichert und wirken nur, solange es aktiviert ist; Passwortfelder bleiben maskiert. Das Deaktivieren behält die Konfiguration — beim erneuten Aktivieren geht es weiter, wo Sie aufgehört haben.
