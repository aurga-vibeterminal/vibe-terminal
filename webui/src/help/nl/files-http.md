`Host with HTTP server` publiceert elke map op de server met één klik als HTTP-bestandsdienst: andere apparaten op hetzelfde netwerk (of via de relay) — een telefoon, een andere computer, een testopstelling — kunnen die map vanuit een browser bekijken en downloaden. Ideaal voor het delen van build-artefacten en logbundels, of voor het overhandigen van bestanden aan apparaten zonder geïnstalleerde client.

## Gebruik

Drie ingangen, alle op een map: het rechtsklikmenu van de bestandslijst, het rechtsklikmenu van een Recent-invoer, en het rechtsklikmenu van het kruimelspoor → `Host with HTTP server`.

Klikken:

1. De server start een statische HTTP-dienst geworteld in die map, met de poort uit het Static HTTP-poortbereik;
2. **Lokale server**: de URL `http://127.0.0.1:port/` wordt rechtstreeks naar het klembord gekopieerd, met een statusmelding "HTTP server: … (gekopieerd naar klembord)";
3. **Externe server** (verbonden via een relay): de client vraagt automatisch een port-forward-mapping aan en kopieert de van buiten bereikbare deel-URL — LAN-beperkingen zijn geen blokkade.

Open die URL in de browser van welk apparaat dan de map bladert en downloadt als een gewone weblijst.

## Draaiende diensten beheren

Elke HTTP-dienst blijft draaien tot hij stopt:

- Open de sectie `Static HTTP` van de Web UI; de lijst `Services` toont elke draaiende dienst en zijn documentwortel;
- Elke invoer heeft een stopknop; het poortbereik wordt op dezelfde pagina geconfigureerd.

> De dienst is een alleen-lezen bestandslijst — geen schrijf-endpoints; mapinhoud is live zichtbaar terwijl die verandert, wat past bij doorlopend geproduceerde logs en artefacten.
