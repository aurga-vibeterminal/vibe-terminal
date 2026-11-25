Vibe Terminal is een op afstand beheerde terminal-hub voor embedded- en serverontwikkeling. Op elke machine waarop je werkt — een ontwikkelmachine, een buildserver, een embedded board — draait één **server**, en **clients** verbinden zich ermee vanuit een desktopapp of de commandoregel. Deze gids behandelt beide kanten: de Web UI waarmee je een server configureert, en de clients die ermee werken.

## Hoe alles samenhangt

- Alles draait op de **server** — terminalsessies leven daar, dus een client sluiten, het netwerk verliezen of vensters herschikken onderbreekt een lopend commando nooit; verbind opnieuw en ga verder waar je gebleven was;
- Clients openen één versleutelde WebSocket-verbinding (`wss://host:port`; `wss://localhost:5153` voor de lokale server) en authenticeren met de **geheime sleutel** van de server; bij het koppelen wordt de sleutel overgedragen via een QR-code of een versleutelde deeltekenreeks (zie Een client verbinden);
- De **Web UI** is de configuratiepagina van de server zelf — profielen, addons, LLM-providers, relay, portforwarding en meer. Het is geen terminal: het dagelijkse werk gebeurt in de desktopclient of `vtermc`.

## Onderdelen

- **Server** — draait op de doelmachine; host de terminalsessies en bedient verbonden clients met bestanden, Git, bureaublad op afstand, spraakinvoer en AI-functies;
- **Desktopclient** — de native werkruimte: terminals met splitsingen, tabbladgroepen en losgekoppelde vensters, een bestandsverkenner en -editor, Git-gereedschap, spraakinvoer, gespreksgeschiedenis van code-agents, bureaublad op afstand en debugging op afstand;
- **`vtermc` CLI** — maak sessies aan en koppel eraan vanuit elke gewone terminal, via SSH of op een headless-machine; deelt de serverlijst met de desktopclient (zie Commandoregel-client);
- **Web UI** — browsergebaseerde instellingen voor de server zelf, geordend in secties in de linkerzijbalk (zie De Web UI gebruiken).

## Pictogram in het systeemvak

Op machines met een bureaubladsessie toont de server zolang hij draait een pictogram in het systeemvak:

![Menu van het systeemvakpictogram — Open brengt een draaiende client naar de voorgrond of start er een, Settings opent de Web UI en het spraakitem start of stopt de spraakherkenningsserver](images/start-tray-menu.jpg)

- **Open** — brengt een draaiende desktopclient naar de voorgrond, of start er een als er geen draait (dubbelklikken op het pictogram doet hetzelfde);
- **Settings** — opent de Web UI van de server in de browser (zie De Web UI gebruiken);
- **Start/Stop Vibe Voice** — start of stopt de spraakherkenningsserver zonder de Web UI te openen, als weerspiegeling van de Start/Stop-knoppen van de sectie `Spraak`; het verschijnt wanneer spraakherkenning is ingeschakeld, en het label volgt de status (bijv. `Stop Vibe Voice (model on device)` zolang het model geladen is);
- **Exit** — sluit de server af.

Zolang mappen over HTTP worden gehost, biedt een item `HTTP Servers` een stopactie met één klik voor elke share (zie Publiceren met HTTP-server).

## Overzicht van functies

De rest van deze gids volgt de hoofdstukken uit de zijbalk:

- **Terminals** — terminals aanmaken, splitsen en organiseren; profielen, WSL en UART-serieel; snelle commando’s en omgevingsvariabelen; terminals aansturen vanuit externe agents met MCP; gesprekken van code-agents bekijken; interactieve prompts automatisch bevestigen;
- **Bestanden** — recente projecten, de bestandsverkenner aan serverzijde, uploaden & downloaden, en een map via HTTP publiceren (zie Bestandsverkenner, Uploaden & downloaden en Publiceren met HTTP-server);
- **Git** — stagen, committen met AI-assistentie, worktrees, interactieve rebase, conflicten oplossen en pushen;
- **Bureaublad op afstand & simulators** — de schermen van de server bekijken en bedienen, Android-apparaten en emulators, en de iOS-simulator;
- **Spraakinvoer** — spraakherkenning aan serverzijde met push-to-talk-dictatie en LLM-nabewerking;
- **Relay** — machines achter NAT of met dynamische IP-adressen bereiken via één publiek toegangspunt;
- **Toegang op afstand** — DDNS-domeinen en TLS-portforwarding voor services die op de server draaien;
- **AI-functies** — LLM-providers als addons, per functie toegewezen, plus MCP-integratie (zie LLM en MCP).

## Waar te beginnen

1. Open de sectie `Server` van de Web UI, stel een `Secret Key` in en koppel een client — Een client verbinden loopt de QR-code- en handmatige installatie met je door;
2. Blader door de secties van de Web UI om profielen, addons en functies in te stellen — zie De Web UI gebruiken;
3. Open een terminal op de server en werk zo nodig door de hoofdstukken hierboven.
