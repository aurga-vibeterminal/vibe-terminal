`Host with HTTP server` veröffentlicht jedes Verzeichnis auf dem Server mit einem Klick als HTTP-Dateidienst: Andere Geräte im selben Netzwerk (oder über das Relay) — ein Telefon, ein anderer Rechner, ein Prüfstand — können dieses Verzeichnis im Browser durchstöbern und herunterladen. Ideal zum Teilen von Build-Artefakten und Protokollpaketen oder zum Übergeben von Dateien an Geräte ohne installierten Client.

## So wird es verwendet

Drei Einstiege, alle auf einem Verzeichnis: das Rechtsklick-Menü der Dateiliste, das Rechtsklick-Menü eines Recent-Eintrags und das Rechtsklick-Menü der Breadcrumbs → `Host with HTTP server`.

Ein Klick darauf:

1. Der Server startet einen statischen HTTP-Dienst mit diesem Verzeichnis als Wurzel; der Port stammt aus dem Static-HTTP-Portbereich;
2. **Lokaler Server**: die URL `http://127.0.0.1:port/` wird direkt in die Zwischenablage kopiert, mit der Statusmeldung „HTTP server: … (in die Zwischenablage kopiert)“;
3. **Remote-Server** (über ein Relay verbunden): der Client fordert automatisch ein Reverse-Proxy-Mapping an und kopiert die extern erreichbare Share-URL — LAN-Beschränkungen sind kein Hindernis.

Öffnen Sie diese URL im Browser eines beliebigen Geräts und das Verzeichnis lässt sich wie eine gewöhnliche Web-Liste durchstöbern und herunterladen.

## Laufende Dienste verwalten

Jeder HTTP-Dienst läuft weiter, bis er gestoppt wird:

- Öffnen Sie den Bereich `Static HTTP` der Web UI; die Liste `Services` zeigt jeden laufenden Dienst und seine Dokumentwurzel;
- Jeder Eintrag hat einen Stopp-Button; der Portbereich wird auf derselben Seite konfiguriert.

> Der Dienst ist eine schreibgeschützte Dateiliste — keine Schreib-Endpunkte; Verzeichnisinhalte sind live sichtbar, während sie sich ändern, was kontinuierlich erzeugte Protokolle und Artefakte gut passt.
