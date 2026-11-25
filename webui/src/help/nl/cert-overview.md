De server beschermt alle TLS-eindpunten die hij aanbiedt — de webinterface, clientverbindingen en doorgestuurde poorten — met een certificaat dat is uitgegeven door zijn eigen **private root-CA**. De pagina `Certificaten & TLS` in de webinterface is waar u die CA beheert, bepaalt voor welke namen het certificaat geldig is, en de CA afgeeft aan clients die de server strikt willen verifiëren.

## Hoe het werkt

- De server genereert een private root-CA precies één keer en bewaart die gedurende de levensduur van de installatie. Alles wordt erdoor ondertekend;
- De CA ondertekent een **bladcertificaat** voor de server met een geldigheid van 90 dagen, dat de hostnamen en IP-adressen dekt die clients daadwerkelijk gebruiken om de server te bereiken;
- Wanneer de hostnamen, netwerkinterfaces of SAN-items van de machine wijzigen, wordt het bladcertificaat automatisch opnieuw uitgegeven — de root-CA verandert nooit, dus clients die haar al vertrouwen blijven de server vertrouwen;
- Een nieuw bladcertificaat wordt via een gedeelde certificaatresolver ingewisseld en zonder herstart herladen: bestaande verbindingen worden nooit verbroken en nieuwe handshakes gebruiken meteen het nieuwste certificaat. Als een verlenging mislukt, blijft het vorige certificaat actief.

![De pagina Certificaten & TLS: bovenaan de kaart Private root-CA, in het midden het SAN-beheer, onderaan de details van het server-bladcertificaat](images/certificates-page.jpg)

## Wat u op de pagina vindt

1. `Private root-CA` — het openbare CA-certificaat downloaden of kopiëren om het te verspreiden naar clients (en hun browsers). De download bevat nooit een privésleutel;
2. `Domeinen & IP-adressen (SAN)` — de lijst met namen waarvoor het bladcertificaat geldig is. Beschreven op de volgende pagina, Domeinen & SAN-beheer;
3. `Server-bladcertificaat` — het actieve certificaat: status, onderwerp, SHA-256-vingerafdruk, geldigheidsperiode en SAN-items.

## Aangepaste TLS-certificaten

Als er een aangepast TLS-certificaat voor de server is geconfigureerd, blijft dat leidend: het beheerde certificaat op deze pagina wordt nog steeds gegenereerd en bijgewerkt, maar vervangt de geconfigureerde aangepaste identiteit niet. Zolang een aangepaste identiteit actief is, toont de pagina een melding.

## Waarom de root-CA vertrouwen?

Clients die alleen versleuteling nodig hebben, werken zonder verdere instelling. Clients die de server willen *verifiëren* — de strikte modus, beschreven op de laatste pagina, Stricte verificatie op clients — hebben het openbare certificaat van de root-CA nodig, en dat moet via een kanaal komen dat u vertrouwt:

> Download de root-CA alleen via de lokale webinterface van deze server of via een vertrouwde tunnel, en controleer de SHA-256-vingerafdruk (getoond op deze pagina) voordat u de strikte verificatie ergens inschakelt.

Dezelfde CA ondertekent ook de certificaten die portforwarding gebruikt (zie de pagina Portforwarding). Zodra een apparaat de root-CA vertrouwt, openen doorgestuurde URL's zonder de browserwaarschuwing *Uw verbinding is niet beveiligd*.
