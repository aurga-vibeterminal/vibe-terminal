Het bladcertificaat is alleen geldig voor de namen die in zijn **Subject Alternative Names (SAN)** staan vermeld. Wanneer een client verbinding maakt, moet de hostnaam of het IP-adres in de URL exact overeenkomen met één van die items — wildcards en prefixmatching worden niet ondersteund. Deze pagina behandelt het beheer van die lijst op de server; de controle aan de clientkant vindt u op de laatste pagina, Stricte verificatie op clients.

## Automatische detectie

De server ontdekt continu onder welke namen hij bereikbaar is — de huidige hostnamen en de adressen van actieve netwerkinterfaces — en zet ze onder `Automatisch ontdekt` op een rij. Deze items volgen de machine en kunnen niet met de hand worden bewerkt: voeg een netwerkinterface toe of wijzig de hostnaam en de lijst volgt bij de volgende verlenging.

## Extra domeinen en IP-adressen

De detectie weet niet alles: DDNS-namen, split-horizon-DNS-items of een adres achter NAT moeten met de hand worden toegevoegd. Voeg ze toe onder `Extra domeinen en IP-adressen`:

1. Typ een hostnaam of IP-adres (bijvoorbeeld `server.home.arpa` of `10.0.0.8`) en druk op `Toevoegen`;
2. Herhaal dit voor elke naam die clients zullen gebruiken. Verwijder een item met de prullenbakknop;
3. Controleer de `Definitieve SAN-preview` — de samengevoegde weergave van ontdekte en handmatige items die het volgende certificaat zal dragen;
4. Druk op `Opslaan & opnieuw uitgeven`. Bij het opslaan wordt een nieuw bladcertificaat van 90 dagen met de nieuwe SAN-lijst ondertekend en zonder herstart herladen — bestaande verbindingen blijven staan en nieuwe handshakes gebruiken meteen het nieuwe certificaat.

## Handmatige heruitgifte en status

Het paneel `Server-bladcertificaat` toont het actieve certificaat: een statusbadge, het onderwerp, de SHA-256-vingerafdruk, de geldigheidsperiode en de SAN-items. De statusbadge betekent:

- `Beschikbaar` — er is een geldig bladcertificaat geladen en in gebruik;
- `Bijwerken mislukt` — de laatste heruitgiftepoging is mislukt; het vorige certificaat blijft actief en de fout wordt op de pagina getoond;
- `Niet gegenereerd` — er bestaan nog geen CA of bladcertificaat; gebruik `CA & certificaat genereren` om beide te maken.

`Bladcertificaat opnieuw uitgeven` ondertekent direct een vers bladcertificaat zonder op een automatische aanleiding te wachten — handig direct na het wijzigen van de hostnaam of de interfaces, of om SAN-wijzigingen die buiten de webinterface zijn gedaan over te nemen.

## De root-CA verspreiden

Clients die de strikte verificatie gaan gebruiken, hebben het openbare certificaat van de root-CA nodig. Haal het op de kaart `Private root-CA`:

- `Root-CA downloaden` slaat het PEM-bestand op;
- `Root-CA kopiëren` zet de PEM op het klembord, klaar om in het CA-veld van een client te plakken.

De CA verandert nooit bij bladcertificaatverlengingen of SAN-bewerkingen — één keer verspreiden is genoeg. Voordat u de strikte verificatie ergens inschakelt, vergelijkt u de SHA-256-vingerafdruk van het certificaat op deze pagina met wat de client heeft ontvangen (zie de laatste pagina, Stricte verificatie op clients).

> De download en het klembord bevatten alleen het openbare CA-certificaat — nooit een privésleutel. Geef de CA alleen aan mensen die uw server moeten kunnen verifiëren.
