Bestandsbeheer laat je vanuit de client rechtstreeks met bestanden op de server werken: de linkeractiviteitenbalk heeft twee invoeren — `Recent` (recente projecten) en de bestandsverkenner — gebruikt naast terminaltabbladen. Alles werkt op de server van de sessie: de lokale machine, of een externe/gedeelde machine via een relay.

## Recent: recente projecten

`Recent` somt de mappen op die je code-agents het meest recent op de huidige server hebben geopend, nieuwste eerst:

- **Klik** op een invoer en de bestandsverkenner schakelt naar die map;
- **Rechtsklik** opent een menu met snelle acties:
  - `Open in VS Code` — de map in VS Code op deze machine openen;
  - `Reveal in File Explorer` — tonen in de bestandsbeheerder van het besturingssysteem;
  - `Copy Full Path` / `Copy Relative Path` — volledig pad, of pad relatief aan de repository;
  - `Search in Folder` — bestanden binnen die map zoeken;
  - `Open in Terminal` — daar een nieuw terminaltabblad openen; `cd Here` — de huidige terminal erheen cd'en;
  - `Host with HTTP server` — de map als HTTP-bestandsdienst publiceren (zie Publiceren met HTTP-server);
  - `Open in App` — openen met een op de server gedetecteerde agent-app.

![Het linkerpaneel — de lijst met recente projecten](images/files-overview.jpg)

## De bestandsverkenner

Het tabblad `Explorer` bladert en beheert het bestandssysteem aan serverzijde: mapnavigatie, boom- en vlakke weergaven, zoeken, aanmaken/hernoemen/verwijderen en andere bestandsbewerkingen, plus uploaden en downloaden door slepen tussen de server en deze machine. Zie Bestandsverkenner en Uploaden & downloaden.
