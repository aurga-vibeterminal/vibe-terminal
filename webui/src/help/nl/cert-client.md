Standaard versleutelt een client elke verbinding, maar controleert niet wie er aan de andere kant zit — dezelfde vertrouw-bij-eerste-gebruikshouding als het accepteren van een zelfondertekend certificaat in een browser. **Strikte verificatie** verandert dat: de client controleert het servercertificaat tegen een root-CA die u aanlevert en weigert te verbinden tenzij de keten, de geldigheid en de SAN's allemaal kloppen. Dit maakt man-in-the-middle-aanvallen op de verbinding praktisch onmogelijk, tegen de prijs van één handmatige stap: de client de root-CA van de server geven.

## Waar het wordt ingesteld

Strikte verificatie wordt per server geconfigureerd in het serverbewerkingspaneel van de client, naast het adres en de geheime sleutel:

1. `Root-CA-certificaat (PEM)` — een veld met meerdere regels voor het openbare root-CA-certificaat van de server;
2. `Stricte CA-verificatie (keten, geldigheid & SAN)` — pas selecteerbaar zodra er een CA is ingevuld.

![Het serverbewerkingspaneel in de desktopclient: meerregelig veld Root-CA-certificaat (PEM) met het selectievakje voor strikte CA-verificatie eronder](images/client-ca-fields.jpg)

## De root-CA bij de client krijgen

Twee wegen, uit dezelfde bron (de pagina `Certificaten & TLS` van de server — zie de pagina Domeinen & SAN-beheer):

- **Kopiëren & plakken** — druk in de webinterface op `Root-CA kopiëren` en plak de PEM in het veld `Root-CA-certificaat (PEM)` van de client;
- **Een deelstring importeren** — deelstrings die de client produceert, bevatten de root-CA naast het adres en de sleutel. Bij het importeren van zo'n deel wordt het CA-veld automatisch vooringevuld; de ontvanger hoeft alleen het selectievakje voor strikte verificatie aan te vinken.

De CA is voor elke client hetzelfde en verandert nooit bij certificaatverlengingen — één keer per server instellen is genoeg.

## Wat de strikte modus controleert

Met het selectievakje aan moet elke verbinding met die server een certificaat tonen dat:

1. Opgaat in de geconfigureerde root-CA — een certificaat dat door iets anders is ondertekend wordt geweigerd;
2. Binnen zijn geldigheidsperiode valt — verlopen of nog niet geldige certificaten worden geweigerd;
3. Een SAN draagt dat overeenkomt met het adres waarmee u verbindt — een certificaat dat voor andere namen is uitgegeven wordt geweigerd.

Als een van de controles mislukt, wordt de verbinding geweigerd met een foutmelding die aangeeft welke controle mislukte. Zonder het selectievakje (of zonder een CA) blijven verbindingen versleuteld, maar worden deze controles overgeslagen.

## De vingerafdruk verifiëren

Voordat u de CA vertrouwt, bevestigt u dat u de echte heeft: de pagina `Certificaten & TLS` toont de SHA-256-vingerafdruk van de root-CA. Vergelijk die met de PEM die u op het punt staat te plakken (of de PEM die in een ontvangen deel is ingebed). Als ze verschillen, schakel de strikte verificatie dan niet in — haal de CA in plaats daarvan via een vertrouwd kanaal.

## Veelvoorkomende fouten

- `SAN komt niet overeen` — u verbindt via een naam of IP-adres dat niet in het certificaat staat. Voeg het op de server toe (zie de pagina Domeinen & SAN-beheer) en geef opnieuw uit;
- `Certificaat verlopen of nog niet geldig` — de klok van de server staat verkeerd, of het bladcertificaat kon niet worden verlengd. Controleer de certificaatstatus in de webinterface;
- `Niet ondertekend door de geconfigureerde CA` — de CA in het veld hoort bij een andere server, of de verbinding wordt onderschept. Vergelijk vingerafdrukken voordat u doorgaat.
