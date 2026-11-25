Vibe Terminal ist eine remote verwaltete Terminal-Hub-Lösung für Embedded- und Server-Entwicklung. Auf jeder Maschine, mit der Sie arbeiten — einem Entwicklungsrechner, einem Build-Server, einem Embedded-Board — läuft ein **Server**; **Clients** verbinden sich über eine Desktop-App oder die Kommandozeile mit ihm. Diese Anleitung deckt beide Seiten ab: die Web UI, die einen Server konfiguriert, und die Clients, die mit ihm arbeiten.

## So hängt alles zusammen

- Alles läuft auf dem **Server** — die Terminal-Sitzungen leben dort: Das Schließen eines Clients, ein Netzwerkabbruch oder das Verschieben von Fenstern unterbricht einen laufenden Befehl nie; verbinden Sie sich erneut und machen Sie dort weiter, wo Sie aufgehört haben;
- Clients wählen eine einzige verschlüsselte WebSocket-Verbindung (`wss://host:port`; `wss://localhost:5153` beim lokalen Server) und authentifizieren sich mit dem **Secret Key** des Servers; beim Pairing wird der Schlüssel per QR-Code oder verschlüsseltem Share-String übergeben (siehe Client verbinden);
- Die **Web UI** ist die Konfigurationsseite des Servers selbst — Profile, Addons, LLM-Anbieter, Relay, Portweiterleitung und mehr. Sie ist kein Terminal: Die tägliche Arbeit erledigen Sie im Desktop-Client oder mit `vtermc`.

## Komponenten

- **Server** — läuft auf der Zielmaschine; beherbergt die Terminal-Sitzungen und stellt verbundenen Clients Dateien, Git, Remote Desktop, Spracheingabe und KI-Funktionen bereit;
- **Desktop-Client** — der native Arbeitsbereich: Terminals mit Splits, Registerkartengruppen und losgelösten Fenstern, ein Datei-Explorer und -Editor, Git-Werkzeuge, Spracheingabe, Gesprächsverlauf der Code-Agents, Remote Desktop und Remote-Debugging;
- **`vtermc`-CLI** — Sitzungen aus jedem schlichten Terminal starten und anhängen, über SSH oder auf einer Headless-Maschine; teilt sich die Serverliste mit dem Desktop-Client (siehe Kommandozeilen-Client);
- **Web UI** — browserbasierte Einstellungen für den Server selbst, in Abschnitten in der linken Seitenleiste organisiert (siehe Die Web UI verwenden).

## Tray-Icon

Auf Maschinen mit einer Desktop-Sitzung zeigt der Server während der Laufzeit ein Tray-Icon:

![Tray-Icon-Menü — Open holt einen laufenden Client in den Vordergrund oder startet einen, Settings öffnet die Web UI, und der Spracheintrag startet oder stoppt den ASR-Server](images/start-tray-menu.jpg)

- **Open** — holt einen laufenden Desktop-Client in den Vordergrund oder startet einen, wenn keiner läuft (ein Doppelklick auf das Icon tut das Gleiche);
- **Settings** — öffnet die Web UI des Servers im Browser (siehe Die Web UI verwenden);
- **Start/Stop Vibe Voice** — startet oder stoppt den ASR-Server, ohne die Web UI zu öffnen, und entspricht den Start-/Stop-Buttons des Bereichs `Sprache`; der Eintrag erscheint, wenn die Spracherkennung aktiviert ist, und die Beschriftung folgt dem Zustand (z. B. `Stop Vibe Voice (model on device)`, während das Modell geladen ist);
- **Exit** — fährt den Server herunter.

Während Verzeichnisse über HTTP freigegeben sind, bietet der Eintrag `HTTP Servers` einen Stopp mit einem Klick für jede Freigabe (siehe Mit HTTP-Server freigeben).

## Funktionsübersicht

Der Rest dieser Anleitung folgt den Kapiteln in der Seitenleiste:

- **Terminals** — Terminals erstellen, teilen und organisieren; Profile, WSL und UART-Seriell; Quick Commands und Umgebungsvariablen; Terminals aus externen Agents über MCP steuern; Gespräche von Code-Agents ansehen; interaktive Eingaben automatisch bestätigen;
- **Dateien** — letzte Projekte, der serverseitige Datei-Explorer, Hoch- & Herunterladen und das Freigeben eines Verzeichnisses über HTTP (siehe Datei-Explorer, Hoch- & Herunterladen und Mit HTTP-Server freigeben);
- **Git** — Stagen, KI-unterstützte Commits, Worktrees, interaktiver Rebase, Konfliktlösung und Push;
- **Remote Desktop & Simulator** — die Bildschirme des Servers sowie Android-Geräte und -Emulatoren und den iOS-Simulator beobachten und steuern;
- **Spracheingabe** — serverseitige Spracherkennung mit Push-to-Talk-Diktat und LLM-Nachbearbeitung;
- **Relay** — Maschinen hinter NAT oder dynamischen IPs über einen einzigen öffentlichen Einstiegspunkt erreichen;
- **Fernzugriff** — DDNS-Domains und TLS-Portweiterleitung für Dienste, die auf dem Server laufen;
- **KI-Funktionen** — LLM-Anbieter als Addons, pro Funktion zugewiesen, plus MCP-Integration (siehe LLM und MCP).

## So beginnen Sie

1. Öffnen Sie den Bereich `Server` der Web UI, setzen Sie einen `Secret Key` und koppeln Sie einen Client — Client verbinden führt durch das Setup per QR-Code und von Hand;
2. Durchsuchen Sie die Abschnitte der Web UI, um Profile, Addons und Funktionen zu konfigurieren — siehe Die Web UI verwenden;
3. Öffnen Sie ein Terminal auf dem Server und arbeiten Sie nach Bedarf die Kapitel oben durch.
