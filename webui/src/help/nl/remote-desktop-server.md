Alles staat in de sectie `Remote Desktop` van de Web UI; de pagina toont de instellingen die op het platform van de server van toepassing zijn.

## Hoofdschakelaar

`Enable Remote Desktop` — eenmaal aan mogen verbonden externe clients het bureaublad van deze machine bekijken en invoer sturen.

## Platforminstellingen

**Windows**

- `Windows Service` — registreert de schermserver als automatisch startende systeemdienst zodat bureaublad op afstand herstarts overleeft zonder ingelogde sessie. `Install` / `Uninstall` triggert op de server een UAC-melding (de knop toont voortgang tijdens het wachten); de statusbadge onderscheidt draaiend / geïnstalleerd / niet geïnstalleerd.
- `Headless-machines (zonder monitor)` — zonder aangesloten (of wakker) beeldscherm houdt Windows geen vastlegbaar bureaublad aan. De pagina linkt de download van het open-source Virtual Display Driver (ARM64 / x64 automatisch gekozen); installeer het zodat er altijd een virtueel bureaublad bestaat.

![Web UI — de sectie Remote Desktop onder Windows (dienst- en headless-kaarten)](images/remote-desktop-server-windows.jpg)

**Linux**

- `Unlock on Connect` — ontgrendelt automatisch de vergrendelde sessie wanneer een stream start (waar de omgeving dat ondersteunt);
- schermvastlegging loopt via de Portal; `Reset Permission` op de kaart `Screen Recording Permission` wist een opgeslagen Portal-toestemming zodat je een ander scherm kunt kiezen.
- `Auto-start on Login` (in de sectie `Server`) — op hosts met een display manager start de server samen met de grafische sessie, zodat de vastlegging de volledige bureaubladomgeving overerft; op headless-hosts start hij in plaats daarvan bij het opstarten, vóór elke login. Om het bureaublad te bereiken wanneer niemand lokaal is ingelogd, schakel ook automatisch inloggen in de display manager in (GDM, SDDM, …); voor de veiligheid kan de automatische schermvergrendeling aan blijven — de sessie en de server blijven draaien terwijl vergrendeld, en `Unlock on Connect` (of `loginctl unlock-session`) opent de sessie op afstand.
![Web UI — de kaart Screen Recording Permission (Linux)](images/remote-desktop-server-permission.jpg)


**macOS**

- `Screen Recording Permission` — het systeem vereist expliciete goedkeuring voor vastlegging. Klik `Grant Permission` om het systeemvenster te openen; `Reset Permission` herstelt uit een vastgelopen toestand.

## Schermstream-gereedschappen (Android-ondersteuning)

`Android SDK directory` en `Android Studio directory` lokaliseren adb en de emulator; laat leeg voor autodetectie. Eenmaal ingesteld toont de pagina het opgeloste pad en de herkomst (auto / env / override); `Reset to Auto-Detect` herstelt de standaard. Zie Android-apparaten voor wat ze mogelijk maken.

## Stream-encoding

- `Encoder` — Auto of een specifieke backend; de server somt bruikbare encoders op en grijst onbeschikbare uit met de reden;
- `Frame rate` — 60 / 30 / 15 fps;
- `Max resolution` — plafond aan de vastlegkant (4K / 2.5K / 2K);
- `Bitrate` — `Auto` schaalt oppervlaktegewijs mee met de resolutie (basistempo tot 1080p, ~4× bij 4K, afgedekt op 40 Mbps); een vaste waarde geldt onmiddellijk voor lopende streams, terwijl encoder / fps / resolutie gelden voor daarna gestarte streams;
- de voet van de kaart toont de nu geldende encoder / fps / resolutie / bitrate.

![Web UI — stream-encoding-instellingen](images/remote-desktop-server-stream.jpg)

Klik `Save` om wijzigingen toe te passen.
