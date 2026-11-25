Bureaublad op afstand laat je de schermen van de server vanuit de desktopclient in realtime bekijken en bedienen: het bureaublad van de machine wordt naar een venster aan jouw kant gestreamd, met je muis en toetsenbord geïnjecteerd — alsof je ervoor zit. Naast bureaubladschermen kunnen Android-emulators en iOS-simulators in telefoon omrande vensters worden bekeken en bestuurd.

![Desktopclient — een bureaublad-op-afstand-venster naast terminals gebruikt](images/remote-desktop-overview-hero.webm)

## Wat het kan

- **Bureaubladschermen** — elke fysieke display van de server streamt naar zijn eigen schaalbare clientvenster;
- **Android-apparaten / -emulators** — verbonden apparaten en AVD-emulators verschijnen als streambronnen, bekijkbaar en aanraakbaar in een telefoon omrand venster, met rotatie en extern opstarten/afsluiten van de emulator;
- **iOS-simulator** (macOS-servers) — de Xcode iOS-simulator bekijken in een telefoon omrand venster met apparaatkeuze en rotatie.

## Hoe het werkt

De server legt het scherm vast en encodeert het met configureerbare encoder, framerates, resolutieplafond en bitrate; de client decodeert naar zwevende vensters en stuurt invoergebeurtenissen terug over de bestaande versleutelde client-serververbinding — geen extra poorten.

De platformvoorbereiding verschilt: Windows kan de schermserver als systeemdienst draaien en heeft op headless-machines een virtuele beeldschermstuurprogramma nodig; Linux gebruikt een Portal-toestemming plus sessie-ontgrendeling; macOS vereist toestemming voor schermopname. Zie Serverinstellingen; Android-apparaten worden behandeld in Android-apparaten, en de clientvensters in De client gebruiken.
