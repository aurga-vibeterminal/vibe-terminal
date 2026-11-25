Verbind een client met deze server via de koppel-QR-code uit de Web UI — of door de gegevens handmatig in te voeren. Alles wat een client nodig heeft (adres, poort, geheime sleutel) staat in de sectie Server.

## De server voorbereiden

- Open de sectie `Server` en zorg dat een `Secret Key` is ingesteld; klik op de dobbelsteenknop om er een te genereren. Een nieuwe sleutel ongeldigt elke gekoppelde client, die opnieuw moet koppelen.
- Vouw het `QR Code`-blok uit en kies de `Network Interface` en `IP Version` (IPv4 / IPv6) die andere machines moeten gebruiken om deze server te bereiken.
- Als er een DDNS-domein is geconfigureerd, kun je `Connection Type` op `DDNS Domain` zetten zodat de QR-code het domein in plaats van een IP-adres bevat.
- De `Pairing Preview` toont de exacte verbindings-URL (`wss://host:port`) die de client zal gebruiken.
- Klik op de `QR code` om een versleutelde deeltekenreeks naar het klembord te kopiëren — clients gebruiken die voor automatisch invullen met één tik.

## Desktopclient

- Klik op de servernaamknop in de titelbalk en kies `Servers` in het menu om het Server Manager-venster te openen.

![Desktopclient — het servermenu geopend via de servernaamknop](images/start-connect-server-menu.jpg)

- Klik op de knop `+` naast de serverlijst om het Add Server-formulier te starten.
- Als de deeltekenreeks op je klembord staat, verschijnt rechtsboven in het formulier een klembordpictogram — klik erop om de tekenreeks te ontsleutelen en `Title`, `Address` en `Secret Key` automatisch in te vullen.
- Vul anders de velden handmatig in: een weergavenaam, de verbindings-URL (`wss://host:port`, IPv6-adressen tussen haakjes) en de geheime sleutel.
- Klik op `Save` en kies daarna de nieuwe server om een sessie te openen.

![Desktopclient — Server Manager met het Add Server-formulier](images/start-connect-add-server.jpg)
