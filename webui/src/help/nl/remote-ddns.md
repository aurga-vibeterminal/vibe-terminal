Breedband thuis en op kantoor komt doorgaans met een dynamisch publiek IP dat zonder aankondiging verandert. DDNS houdt een vast domein gericht op het huidige IP van de machine — clients en andere servers hoeven alleen de naam te onthouden, nooit het adres.

## Hoe het werkt

![DDNS-architectuur — de manager detecteert periodiek het lokale IP, de addon actualiseert het DNS-record via de provider-API, clients verbinden via het domein](diagram/ddns)

De hele pijplijn draait automatisch op de achtergrond van de server:

1. De **DDNS-manager** ververs op een vaste cyclus (standaard elke 5 minuten, instelbaar via de omgevingsvariabele `TERMINAL_SERVER_DDNS_POLL_INTERVAL_SECS`); het opslaan van de configuratie triggert een onmiddellijke verversing, en er draait niets zolang de dienst uitgeschakeld is;
2. **Lokale IP-detectie** leest eerst de gekozen netwerkinterface, valt terug op route-gebaseerde detectie, en filtert het resultaat op de instelling `IP Family` (IPv4 / IPv6 / dual);
3. De **DDNS-addon** — een sandboxed JS-script — ontvangt de ontsleutelde configuratie en het huidige IP en roept de API van de DNS-provider over HTTP aan. De ingebouwde Cloudflare-addon zoekt bijvoorbeeld de zone op domein en herschrijft de A / AAAA-records naar het huidige IP, en verwijdert verouderde records wanneer een adresfamilie verdwijnt;
4. **Clients** herleiden het domein naar het nieuwste IP en verbinden via de naam.

## Configuratie

Open de sectie `DDNS` van de Web UI:

1. Zet `Enable` aan. Je kunt de dienst ook uit laten terwijl je alles invult — een uitgeschakelde dienst slaat zijn configuratie op maar draait nooit;
2. kies een provider onder `Provider`; `Cloudflare DDNS` is ingebouwd;
3. vul de providervelden in (Cloudflare-voorbeeld):
   - `Email` — het e-mailadres van het Cloudflare-account;
   - `API Token` — een token met Zone → DNS → Edit-rechten;
   - `Domain` — het hoofddomein van de zone, bijv. `example.com`;
   - `Subdomain` — de recordnaam binnen de zone, bijv. `sub` voor `sub.example.com`; laat leeg voor de zonewortel;
4. kies bij `Network Interface & IP` de `Interface` en de `IP Family`. `Dual Stack` vereist dat de interface zowel een IPv4- als een IPv6-adres heeft;
5. klik `Save`, daarna `Sync Now` om te verifiëren. `Sync Status` toont het laatst gepushte IP en het synchronisatietijdstip, samen met de foutmelding van de addon bij mislukking.

## Beveiliging

Providerreferenties (API-tokens) worden voor opslag versleuteld (`enc-v2:`) en alleen in geheugen ontsleuteld terwijl de addon draait; addons draaien in een sandboxed JS-runtime met uitgaand HTTP als enige capaciteit.

## Providers uitbreiden

Providers zijn addons: een JSON-manifest (dat de configuratievelden declareert die de Web UI tot een formulier maakt) plus een JS-script dat `run()` blootstelt. De ingebouwde `cloudflare-ddns` is de referentie-implementatie — aangepaste providers kunnen als zip in de sectie `Addons` worden geüpload.

## Samen met andere functies

- Zodra het domein herleidt, kan de QR-code van de sectie `Server` `Connection Type` op `DDNS Domain` zetten, zodat clients een domeinverbinding scannen;
- andere servers kunnen `wss://your-domain:port` in hun `Relay Address` zetten om de relayketen via het domein te draaien (zie het Relay-hoofdstuk).
