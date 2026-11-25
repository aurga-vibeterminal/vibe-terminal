Standardmäßig verschlüsselt ein geteilter Server seine Upstream-Verbindung zum Relay, prüft aber nicht, wer auf der anderen Seite ist — dieselbe Vertrauen-beim-ersten-Gebrauch-Haltung wie das Akzeptieren eines selbstsignierten Zertifikats im Browser. **Striktes Relay-TLS** ändert das: Der geteilte Server prüft das Zertifikat des Relays gegen eine Root-CA Ihrer Wahl und verweigert die Verbindung, sofern Kette, Gültigkeit und Hostname nicht alle in Ordnung sind. Das macht Man-in-the-Middle-Angriffe auf die Relay-Verbindung praktisch unmöglich — zum Preis eines manuellen Schritts: dem geteilten Server die Root-CA des Relays zu geben. Es wendet dieselbe Idee wie die vorherige Seite, Strikte Verifizierung auf Clients, auf die Upstream-Relay-Verbindung des geteilten Servers an.

## Wo es eingerichtet wird

Die strikte Verifizierung wird auf dem geteilten Server konfiguriert — der Maschine, deren `Relay Address` auf das Relay zeigt. Öffnen Sie in deren Web-Oberfläche den Bereich `Server`: Unter dem `Relay Secret Key` liegt eine Zeile `Striktes Relay-TLS`, die den aktuellen `On`-/`Off`-Zustand zeigt. Klappen Sie sie auf:

1. `Root-CA des Upstream-Relays` — ein mehrzeiliges Feld für das öffentliche Root-CA-Zertifikat des Relays;
2. Ein `Off`/`On`-Umschalter — `On` lässt sich erst wählen, wenn die PEM als gültiges Zertifikat gelesen werden kann.

![Web UI — der aufgeklappte Block Striktes Relay-TLS im Bereich Server: Feld Root-CA des Upstream-Relays mit Off/On-Umschalter und Apply-Button](images/cert-relay-strict-tls.jpg)

Relay-Einstellungen werden nie mitten im Editieren gespeichert: Prüfen Sie die Werte und drücken Sie `Apply`. Das Speichern verbindet das Upstream-Relay sofort mit den neuen Einstellungen neu.

## Die Root-CA zum geteilten Server bringen

Zwei Wege, aus derselben Quelle (die Web-Oberfläche des Relays selbst):

- **Freigabe-String einfügen** — klicken Sie in der Web UI des Relay-Servers auf den `QR code`. Der Freigabe-String bettet die Root-CA des Relays zusammen mit Adresse und Schlüssel ein; wird er beim geteilten Server ins Feld `Relay Address` eingefügt, wird auch das CA-Feld automatisch befüllt (siehe die Einrichtungsanleitung unter Relay). Der strikte Modus bleibt aus, bis Sie die CA geprüft und selbst eingeschaltet haben;
- **Kopieren & Einfügen** — drücken Sie auf der Relay-Seite `Zertifikate & TLS` auf `Root-CA kopieren` und fügen Sie die PEM in das Feld `Root-CA des Upstream-Relays` ein.

Die CA ist für jeden geteilten Server dieselbe und ändert sich über Zertifikatserneuerungen hinweg nie — einmal pro Relay eintragen genügt.

Bevor Sie der CA vertrauen, bestätigen Sie, dass Sie die echte haben: Die Relay-Seite `Zertifikate & TLS` zeigt den SHA-256-Fingerabdruck der Root-CA. Vergleichen Sie ihn mit der PEM, die Sie gerade einfügen wollen. Weichen sie ab, aktivieren Sie die strikte Verifizierung nicht — besorgen Sie die CA stattdessen über einen vertrauenswürdigen Kanal.

## Was der strikte Modus prüft

Mit aktiviertem Umschalter muss die Upstream-`RelayPrimary`-Verbindung ein Zertifikat vorweisen, das:

1. zur konfigurierten Root-CA führt — ein von etwas anderem signiertes Zertifikat, öffentliche Systemwurzeln eingeschlossen, wird abgelehnt;
2. innerhalb seines Gültigkeitszeitraums liegt — abgelaufene oder noch nicht gültige Zertifikate werden abgelehnt;
3. ein SAN trägt, das zum Host der `Relay Address` passt, zu der Sie sich verbinden — ein für andere Namen ausgestelltes Zertifikat wird abgelehnt.

Eine Grenze bleibt: Nur die Upstream-Relay-Verbindung wird verifiziert. Terminal- und Anfrage-Rückverbindungen behalten die bestehende Zertifikatspolitik.

Schlägt eine der Prüfungen fehl, wird die Verbindung verweigert und im Hintergrund mit wachsenden Abständen erneut versucht; der geteilte Server bleibt über das Relay unerreichbar, bis die Prüfung besteht oder Sie den strikten Modus wieder ausschalten.

## Häufige Fehlerszenarien

- `Strikte TLS-Verifizierung erfordert ein Root-CA-PEM` — der Umschalter ist an, aber das CA-Feld ist leer. Fügen Sie zuerst die PEM ein;
- `SAN stimmt nicht überein` — Sie verbinden sich über einen Namen oder eine IP, die nicht im Zertifikat des Relays steht. Fügen Sie ihn auf der Relay-Seite `Zertifikate & TLS` hinzu (siehe die Seite Domains & SAN verwalten) und stellen Sie neu aus;
- `Zertifikat abgelaufen oder noch nicht gültig` — die Uhr des Relays geht falsch, oder das Blattzertifikat konnte nicht erneuert werden. Prüfen Sie den Zertifikatsstatus auf dem Relay;
- `Nicht von der konfigurierten CA signiert` — die CA im Feld gehört zu einem anderen Relay, oder die Verbindung wird abgefangen. Vergleichen Sie Fingerabdrücke, bevor Sie fortfahren.
