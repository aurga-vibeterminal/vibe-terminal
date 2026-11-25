Standaard versleutelt een gedeelde server zijn upstream-verbinding met de relay, maar controleert niet wie er aan de andere kant zit — dezelfde vertrouw-bij-eerste-gebruikshouding als het accepteren van een zelfondertekend certificaat in een browser. **Strikt relay-TLS** verandert dat: de gedeelde server controleert het certificaat van de relay tegen een root-CA die u aanlevert en weigert te verbinden tenzij de keten, de geldigheid en de hostnaam allemaal kloppen. Dit maakt man-in-the-middle-aanvallen op de relay-link praktisch onmogelijk, tegen de prijs van één handmatige stap: de gedeelde server de root-CA van de relay geven. Het past hetzelfde idee als de vorige pagina, Stricte verificatie op clients, toe op de upstream-relay-link van de gedeelde server.

## Waar het wordt ingesteld

Strikte verificatie wordt geconfigureerd op de gedeelde server — de machine waarvan het `Relay Address`-veld naar de relay wijst. Open in zijn webinterface de sectie `Server`: onder de `Relay Secret Key` staat een rij `Strikt relay-TLS` die de huidige `On`/`Off`-toestand toont. Klap hem uit:

1. `Root-CA van upstream-relais` — een veld met meerdere regels voor het openbare root-CA-certificaat van de relay;
2. Een `Off`/`On`-schakelaar — `On` is pas te kiezen zodra de PEM als geldig certificaat te lezen is.

![Web UI — het uitgeklapte blok Strikt relay-TLS in de sectie Server: veld Root-CA van upstream-relais met de Off/On-schakelaar en de knop Apply](images/cert-relay-strict-tls.jpg)

Relay-instellingen worden nooit midden in het bewerken opgeslagen: controleer de waarden en klik op `Apply`. Opslaan verbindt direct opnieuw met de upstream-relay met de nieuwe instellingen.

## De root-CA bij de gedeelde server krijgen

Twee wegen, uit dezelfde bron (de webinterface van de relay zelf):

- **Deeltekenreeks plakken** — klik in de webinterface van de relayserver op de `QR code`. De deeltekenreeks bevat de root-CA van de relay naast het adres en de sleutel; plak hem in het veld `Relay Address` van de gedeelde server en ook het CA-veld wordt automatisch ingevuld (zie de Installatiehandleiding onder Relay). De strikte modus blijft uit tot u de CA hebt gecontroleerd en hem zelf aanzet;
- **Kopiëren & plakken** — druk op `Root-CA kopiëren` op de pagina `Certificaten & TLS` van de relay en plak de PEM in het veld `Root-CA van upstream-relais`.

De CA is voor elke gedeelde server hetzelfde en verandert nooit bij certificaatverlengingen — één keer per relay instellen is genoeg.

Voordat u de CA vertrouwt, bevestigt u dat u de echte heeft: de pagina `Certificaten & TLS` van de relay toont de SHA-256-vingerafdruk van de root-CA. Vergelijk die met de PEM die u op het punt staat te plakken. Als ze verschillen, schakel de strikte verificatie dan niet in — haal de CA in plaats daarvan via een vertrouwd kanaal.

## Wat de strikte modus controleert

Met de schakelaar op `On` moet de upstream-`RelayPrimary`-verbinding een certificaat tonen dat:

1. Opgaat in de geconfigureerde root-CA — een certificaat dat door iets anders is ondertekend, waaronder publieke systeemwortels, wordt geweigerd;
2. Binnen zijn geldigheidsperiode valt — verlopen of nog niet geldige certificaten worden geweigerd;
3. Een SAN draagt die overeenkomt met de host van het `Relay Address` waarmee u verbindt — een certificaat dat voor andere namen is uitgegeven wordt geweigerd.

Eén beperking: alleen de upstream-relayverbinding wordt geverifieerd. Terminal- en verzoek-terugverbindingen houden het bestaande certificaatbeleid aan.

Als een van de controles mislukt, wordt de verbinding geweigerd en op de achtergrond met toenemende tussenpozen opnieuw geprobeerd; de gedeelde server blijft via de relay onbereikbaar tot de controle slaagt of u de strikte modus weer uitzet.

## Veelvoorkomende fouten

- `Strikte TLS-verificatie vereist een root-CA-PEM` — de schakelaar staat aan maar het CA-veld is leeg. Plak eerst de PEM;
- `SAN komt niet overeen` — u benadert de relay via een naam of IP-adres dat niet in het certificaat staat. Voeg het op de pagina `Certificaten & TLS` van de relay toe (zie de pagina Domeinen & SAN-beheer) en geef opnieuw uit;
- `Certificaat verlopen of nog niet geldig` — de klok van de relay staat verkeerd, of het bladcertificaat kon niet worden verlengd. Controleer de certificaatstatus op de relay;
- `Niet ondertekend door de geconfigureerde CA` — de CA in het veld hoort bij een andere relay, of de verbinding wordt onderschept. Vergelijk vingerafdrukken voordat u doorgaat.
