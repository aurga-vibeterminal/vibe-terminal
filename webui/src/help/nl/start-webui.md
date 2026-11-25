De Web UI is de eigen configuratiepagina van de server: elke serverfunctie is ondergebracht in een sectie, vermeld in de linkerzijbalk. Het is geen terminal — het dagelijkse werk gebeurt in de desktopclient of `vtermc` (zie Overzicht). Deze pagina legt de indeling en het algemene gedrag uit; de functiehoofdstukken behandelen de instellingen van elke sectie in detail.

## Indeling

- **Zijbalk** — de sectielijst (hieronder), het helpictogram naast de servernaam en de app-versie onderaan; op smalle schermen schuift hij vanaf links naar binnen en de hamburgerknop in de kop schakelt hem in en uit;
- **Kop** — de titel van de huidige sectie plus, rechtsboven, de taalkiezer en de licht/donker-schakelaar (op smalle schermen staat daar ook het helpictogram);
- **Inhoud** — het formulier van de geselecteerde sectie, gecentreerd en scrollbaar.

## Secties

In de volgorde van de zijbalk:

- `Server` — poort-, TLS-certificaat- en authenticatie-instellingen: de `Geheime sleutel`, de koppel-`QR-code`, het relay-adres en de schakelaar `Auto-start on Login` staan allemaal hier (zie Een client verbinden en Installatiehandleiding). Op Linux legt de hint bij automatisch starten ook het automatisch inloggen van de display manager uit om het bureaublad te bereiken vóór een lokale login (zie Serverinstellingen in Bureaublad op afstand & simulators);
- `Terminal` — terminalanalyse: promptdetectie, automatische bevestiging en LLM-classificatie (zie Automatisch bevestigen);
- `UART` — seriële terminalprofielen op basis van lokale UART-poorten (zie UART-serieel);
- `Add-ons` — geïnstalleerde add-ons: inschakelen, bewerken of uploaden (LLM-providers zijn add-ons — zie LLM);
- `LLM` — wijst aan elke AI-functie een LLM-provider toe (zie LLM);
- `Spraak` — spraakherkenningsprovider, model en transcriptie-instellingen (zie Serverinstellingen in Spraakinvoer);
- `Omgeving` — verzamelingen omgevingsvariabelen in een boom van groepen (zie Omgevingsvariabelen);
- `MCP` — de terminal-MCP-server die aan code-agents wordt aangeboden (zie Terminal MCP);
- `DDNS` — DDNS-provider en synchronisatie-instellingen (zie DDNS);
- `Portforwarding` — de pool met geforwarde poorten en de actieve TLS-forwards (zie Portforwarding);
- `Static HTTP` — mappen die over HTTP worden gehost en de actieve bestandsdelingen (zie Publiceren met HTTP-server);
- `Extern bureaublad` — opname-, invoer- en streaminstellingen voor extern bureaublad (zie Serverinstellingen in Bureaublad op afstand & simulators);
- `LLM-statistieken` — tokenverbruik, aantal aanvragen en prestatiestatistieken voor alle LLM-functies.

## Opslaan & tips

- De meeste secties slaan kort na een bewerking automatisch op — een opslaan-indicator en eventuele fouten verschijnen onderaan de sectie;
- `MCP`, `DDNS` en `Extern bureaublad` bewaren wijzigingen tot op `Wijzigingen opslaan` is gedrukt: zolang er wijzigingen in behandeling zijn, verschijnt naast de knoppen een hint `Niet-opgeslagen wijzigingen`, en `Resetten` verwerpt ze;
- Geopend vanuit de mobiele client volgt de Web UI de taal en het thema van de app en verbergt daarbij de eigen taal- en themaknoppen.
