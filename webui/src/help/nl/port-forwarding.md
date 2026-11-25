Portforwarding publiceert een HTTP-dienst die op de servermachine draait — een dev-server, een apparaatdashboard, alles wat op een gewoon `http://localhost:poort`-adres luistert — op een eigen TLS-poort, zodat elk apparaat dat de server kan bereiken hem in een browser opent als `https://<server-adres>:<poort>/`.

## Hoe het werkt

- Een forward koppelt een **doelpoort** — de HTTP-dienst op deze machine — aan een **geforwarde poort** die wordt toegewezen uit het geconfigureerde poortbereik (standaard `62000`–`62999`);
- De geforwarde poort spreekt TLS: al het verkeer is versleuteld met het **zelfondertekende certificaat** van de server — niets reist in platte tekst, zelfs niet over een onvertrouwd netwerk;
- Elk apparaat dat de server kan bereiken — op het LAN, of vanaf internet zodra het poortbereik op de router is geforward — opent `https://<server-adres>:<geforwardePoort>/` en landt op de lokale dienst.

Forwards worden op aanvraag aangemaakt: de `Host with HTTP server`-bestandsshare op een externe (gerelayde) server vraagt er automatisch een aan en overhandigt je de kant-en-klare URL. Handmatige configuratie is niet nodig.

## Forwards beheren

![Web UI — de sectie Port Forwarding: poortbereikconfiguratie en de lijst met actieve forwards](images/port-forwarding-webui.jpg)

1. Open de sectie `Port Forwarding` van de Web UI van de server.
2. `Port Range Configuration` begrenst de pool waaruit geforwarde poorten worden toegewezen; wijzigingen worden automatisch opgeslagen. Verklein het bereik wanneer je firewall of router maar een paar poorten forwardt.
3. De livelijst toont elke forward als `:geforwardePoort → :doelPoort` met het aantal actieve verbindingen.
4. De stopknop van een forward breekt de listener onmiddellijk af en verbreekt de actieve verbindingen; de volgende share wijst een nieuwe poort toe. `Refresh` leest de lijst opnieuw in.

## De certificaatwaarschuwing

Forwards zijn versleuteld met het eigen zelfondertekende certificaat van de server — hetzelfde certificaat dat de Web UI en clientverbindingen beveiligt. Browsers kunnen het niet verifiëren tegen een certificaatautoriteit, dus toont het eerste bezoek aan een geforwarde URL een privacywaarschuwing (*Uw verbinding is niet privé*). Dit is verwacht gedrag en veilig om door te klikken: de versleuteling zelf is niet aangetast, alleen de herkomst van het certificaat kan niet machinaal worden geverifieerd. Om de waarschuwing definitief uit te schakelen, importeer je het certificaat van de server in de vertrouwde opslag van je apparaat.

![Browser — de waarschuwing voor het onvertrouwde zelfondertekende certificaat; ga verder via Geavanceerd → Doorgaan om de pagina te openen](images/port-forwarding-cert-warning.jpg)

## Chrome op macOS toont `ERR_ADDRESS_UNREACHABLE`

Op macOS 15 (Sequoia) en later hebben apps van derden expliciete toestemming nodig om apparaten op het lokale netwerk te bereiken — elk `192.168.x.x`-, `10.x.x.x`- of vergelijkbaar adres. Safari is als systeem-app vrijgesteld, dus een geforwarde URL kan prima openen in Safari terwijl Chrome `ERR_ADDRESS_UNREACHABLE` rapporteert voordat TLS überhaupt start. Dit is geen certificaatprobleem en geen probleem met de forward zelf.

Om Chrome toegang te verlenen:

1. Open `Systeeminstellingen` → `Privacy en beveiliging` → `Lokaal netwerk`.
2. Zoek `Google Chrome` in de lijst en zet hem **aan**.
3. Sluit Chrome volledig af (`Cmd+Q`, niet alleen de vensters sluiten) en start hem opnieuw.

Als de schakelaar al aan staat, zet hem dan uit en weer aan — hij kan vastlopen na een Chrome- of macOS-update. Als de fout aanhoudt, controleer dan op een VPN- of proxy-extensie in Chrome die privé-adresbereiken niet routeert.

## Toegangsbeheer

Wat een doorgestuurde poort beschermt, hangt af van wat erachter draait:

- **Ingebouwde shares vereisen een token.** De deellinks die de client genereert — een `Host with HTTP server`-share (`?share_token=…`) en de Settings-pagina (`?ticket=…`) — bevatten een toegangstoken. Een verbinding zonder geldig token en zonder geldige sessie wordt verbroken voordat er ook maar iets wordt geserveerd. Bij het eerste bezoek wisselt de browser het token in voor een beveiligde sessiecookie die alleen voor die ene dienst geldt, en verdwijnt het token uit de URL — het blijft niet achter in de adresbalk of de geschiedenis.

- **De tokens zijn eenmalig en kortlevend.** Een Settings-ticket werkt precies één keer en verloopt binnen een minuut; zodra een share stopt of herstart, werken eerdere links en sessies niet meer. Behandel de volledige link als een toegangsbewijs — wie hem heeft, kan bij de dienst — en deel hem alleen met mensen die je vertrouwt.

- **Eigen diensten brengen hun eigen authenticatie mee.** Een forward naar een andere lokale dienst versleutelt het verkeer met het zelfondertekende certificaat van de server, maar voegt zelf geen authenticatie toe: alles wat de dienst serveert — pagina's zowel als API's — is bereikbaar voor iedereen die de poort kan bereiken. Zet authenticatie op gevoelige diensten voordat je ze blootstelt.
