Naast bureaubladschermen kan bureaublad op afstand Android-apparaten streamen — echte telefoons via adb en AVD-emulators. De client toont ze in een telefoon omrand venster met aanraakinvoer, rotatie en externe beheersing van de emulatorlevenscyclus.

## Vereisten

De server moet de Android-toolchain vinden:

- **Android SDK** (levert `adb`) — vereist voor zowel apparaten als emulators;
- **Android Studio** — vereist om AVD-emulators te starten.

Beide worden standaard autodetecteerd. Als detectie faalt of de gereedschappen buiten de standaardlocaties leven, stel dan `Android SDK directory` / `Android Studio directory` in onder `Remote Desktop` → `Screen streaming tools`; wijzigingen gelden direct (hot toegepast, zonder server-herstart).

## Apparaatbronnen

Met de paden op orde somt de server de beschikbare Android-bronnen automatisch op:

- **Verbonden echte apparaten** — sluit aan via USB met USB-debugging ingeschakeld; alles wat `adb devices` ziet kan streamen;
- **AVD-emulators** — draaiende emulators verschijnen uit zichzelf; je kunt ook een gekozen AVD op verzoek vanaf de client opstarten (niet eerst handmatig hoeven starten) en hem op afstand afsluiten.

Elke bron krijgt zijn eigen invoer in de SCREENS-balk van de client.

## Oriëntatie

De roteerknop van het emulatorvenster stelt de systeem-`user_rotation` in via `adb` — de eigen weergaverichting van het apparaat verandert, en het venster past zich aan de nieuwe resolutie aan, in zowel staand als liggend.

## Clientzijde

Open het tabblad `Debug` in het linkerpaneel van de desktopclient en klik op de knop `Android Simulator` in de SCREENS-balk (verschijnt alleen als de server Android-bronnen blootstelt) om het emulatorvenster te openen; de apparaatkeuze wisselt het uiterlijk van de telefoonlijst en `Rotate` spiegelt de oriëntatie. Zie De client gebruiken.

> Aanrak-injectie op echte apparaten gebruikt het invoerkanaal van adb (Android 4.4+); sommige door fabrikanten aangepaste ROM's beperken het, in dat geval is het apparak alleen-kijken.
