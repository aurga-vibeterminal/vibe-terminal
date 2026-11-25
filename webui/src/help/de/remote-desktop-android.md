Neben Desktop-Bildschirmen kann Remote Desktop Android-Geräte streamen — echte Telefone über adb und AVD-Emulatoren. Der Client zeigt sie in einem telefongeformten Fenster mit Berühreingabe, Drehung und ferngesteuerter Emulator-Lebenszyklussteuerung.

## Voraussetzungen

Der Server muss die Android-Toolchain finden:

- **Android SDK** (liefert `adb`) — für Geräte wie Emulatoren erforderlich;
- **Android Studio** — erforderlich, um AVD-Emulatoren zu starten.

Beide werden standardmäßig automatisch erkannt. Schlägt die Erkennung fehl oder liegen die Werkzeuge außerhalb der Standardorte, setzen Sie `Android SDK directory` / `Android Studio directory` unter `Remote Desktop` → `Screen streaming tools`; Änderungen greifen sofort (hot angewendet, kein Server-Neustart).

## Gerätequellen

Sind die Pfeade gesetzt, zählt der Server verfügbare Android-Quellen automatisch auf:

- **Angeschlossene echte Geräte** — per USB mit aktiviertem USB-Debugging anschließen; alles, was `adb devices` sieht, kann streamen;
- **AVD-Emulatoren** — laufende Emulatoren erscheinen von selbst; Sie können auch einen gewählten AVD vom Client auf Abruf starten (nicht erst von Hand starten) und ferngesteuert herunterfahren.

Jede Quelle erhält ihren eigenen Eintrag in der SCREENS-Leiste des Clients.

## Ausrichtung

Der Drehen-Button des Emulator-Fensters setzt die System-`user_rotation` über `adb` — die eigene Anzeigerichtung des Geräts ändert sich, und das Fenster passt sich der neuen Auflösung an, Hoch- wie Querformat.

## Client-Seite

Öffnen Sie den Tab `Debug` im linken Panel des Desktop-Clients und klicken Sie in der SCREENS-Leiste auf den Button `Android Simulator` (erscheint nur, wenn der Server Android-Quellen freigibt), um das Emulator-Fenster zu öffnen; die Geräteauswahl wechselt das Aussehen des Telefonrahmens, und `Rotate` dreht die Ausrichtung. Siehe Client verwenden.

> Die Berühreinjektion auf echten Geräten nutzt den Eingabekanal von adb (Android 4.4+); manche herstellereigene ROMs schränken sie ein, in dem Fall ist das Gerät nur ansehbar.
