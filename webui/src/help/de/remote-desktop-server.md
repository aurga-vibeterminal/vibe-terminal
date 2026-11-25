Alles lebt im Bereich `Remote Desktop` der Web UI; die Seite zeigt die Einstellungen, die für die Plattform des Servers gelten.

## Hauptschalter

`Enable Remote Desktop` — einmal an, dürfen verbundene Remote-Clients den Desktop dieser Maschine betrachten und Eingaben senden.

## Plattformeinstellungen

**Windows**

- `Windows Service` — registriert den Bildschirmserver als selbststartenden Systemdienst, sodass Remote Desktop Neustarts ohne angemeldete Sitzung übersteht. `Install` / `Uninstall` löst auf dem Server eine UAC-Abfrage aus (der Button zeigt den Fortschritt während des Wartens); das Status-Abzeichen unterscheidet laufend / installiert / nicht installiert.
- `Headless machines (no monitor)` — ohne angeschlossenen (oder wachen) Bildschirm hält Windows keinen erfassbaren Desktop vor. Die Seite verlinkt den Download des Open-Source Virtual Display Driver (ARM64 / x64 automatisch gewählt); installieren Sie ihn, damit immer ein virtueller Desktop existiert.

![Web UI — der Bereich Remote Desktop unter Windows (Dienst- und Headless-Karten)](images/remote-desktop-server-windows.jpg)

**Linux**

- `Unlock on Connect` — entsperrt die gesperrte Sitzung automatisch beim Start eines Streams (sofern die Umgebung es unterstützt);
- Die Bilderfassung läuft über das Portal; `Reset Permission` auf der Karte `Screen Recording Permission` löscht eine gespeicherte Portal-Zustimmung, sodass Sie einen anderen Bildschirm wählen können.
- `Auto-start on Login` (im Bereich `Server`) — auf Hosts mit Display-Manager startet der Server zusammen mit der grafischen Sitzung, sodass die Erfassung die vollständige Desktop-Umgebung erbt; auf Headless-Hosts startet er stattdessen beim Systemstart vor jeder Anmeldung. Um den Desktop zu erreichen, wenn sich niemand lokal angemeldet hat, aktivieren Sie zusätzlich die automatische Anmeldung im Display-Manager (GDM, SDDM, …); aus Sicherheitsgründen kann die automatische Bildschirmsperre aktiv bleiben — Sitzung und Server laufen im gesperrten Zustand weiter, und `Unlock on Connect` (oder `loginctl unlock-session`) öffnet die Sitzung aus der Ferne.
![Web UI — die Karte Screen Recording Permission (Linux)](images/remote-desktop-server-permission.jpg)


**macOS**

- `Screen Recording Permission` — das System verlangt eine ausdrückliche Freigabe für die Erfassung. `Grant Permission` öffnet den Systemdialog; `Reset Permission` erholt sich aus einem hängenden Zustand.

## Bildschirm-Streaming-Werkzeuge (Android-Unterstützung)

`Android SDK directory` und `Android Studio directory` lokalieren adb und den Emulator; für automatische Erkennung leer lassen. Einmal gesetzt, zeigt die Seite den aufgelösten Pfad und seine Herkunft (auto / env / override); `Reset to Auto-Detect` stellt den Standard wieder her. Was diese freischalten, siehe Android-Geräte.

## Stream-Kodierung

- `Encoder` — Auto oder ein bestimmtes Backend; der Server listet nutzbare Encoder und blendet nicht verfügbare mit Begründung aus;
- `Frame rate` — 60 / 30 / 15 fps;
- `Max resolution` — Obergrenze auf der Erfassungsseite (4K / 2.5K / 2K);
- `Bitrate` — `Auto` skaliert flächenproportional mit der Auflösung (Basisrate bis 1080p, ~4× bei 4K, bei 40 Mbps gedeckelt); ein fester Wert gilt sofort für laufende Streams, während Encoder / fps / Auflösung für danach gestartete Streams gelten;
- Die Kartenfußzeile zeigt den aktuell wirksamen Encoder / fps / Auflösung / Bitrate.

![Web UI — Stream-Kodierungseinstellungen](images/remote-desktop-server-stream.jpg)

Klicken Sie `Save`, um Änderungen anzuwenden.
