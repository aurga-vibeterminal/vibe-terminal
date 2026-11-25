Das Blattzertifikat ist nur für die Namen gültig, die in seinen **Subject Alternative Names (SAN)** aufgeführt sind. Verbindet sich ein Client, muss der Hostname oder die IP in seiner URL exakt einem dieser Einträge entsprechen — Wildcards und Präfix-Matching werden nicht unterstützt. Diese Seite behandelt die Verwaltung dieser Liste auf dem Server; die Prüfung auf Client-Seite finden Sie auf der letzten Seite, Strikte Verifizierung auf Clients.

## Automatische Erkennung

Der Server erkennt fortlaufend, unter welchen Namen er erreichbar ist — aktuelle Hostnamen und die Adressen aktiver Netzwerkschnittstellen — und listet sie unter `Automatisch erkannt` auf. Diese Einträge folgen der Maschine und lassen sich nicht von Hand bearbeiten: Fügen Sie eine Netzwerkschnittstelle hinzu oder ändern Sie den Hostnamen, passt sich die Liste bei der nächsten Erneuerung an.

## Zusätzliche Domains und IP-Adressen

Die Erkennung weiß nicht alles: DDNS-Namen, Split-Horizon-DNS-Einträge oder eine Adresse hinter NAT müssen von Hand eingetragen werden. Fügen Sie sie unter `Zusätzliche Domains und IP-Adressen` hinzu:

1. Hostnamen oder IP-Adresse eingeben (zum Beispiel `server.home.arpa` oder `10.0.0.8`) und `Hinzufügen` drücken;
2. Für jeden Namen, den Clients verwenden werden, wiederholen. Einträge über den Papierkorb-Button entfernen;
3. Die `Finale SAN-Vorschau` prüfen — die zusammengeführte Ansicht aus erkannten und manuellen Einträgen, die das nächste Zertifikat tragen wird;
4. `Speichern & neu ausstellen` drücken. Beim Speichern wird ein neues 90-Tage-Blattzertifikat mit der neuen SAN-Liste signiert und ohne Neustart geladen — bestehende Verbindungen bleiben bestehen, und neue Handshakes verwenden sofort das neue Zertifikat.

## Manuelle Neuausstellung und Status

Das Panel `Server-Blattzertifikat` zeigt das aktive Zertifikat: ein Status-Badge, den Antragsteller, den SHA-256-Fingerabdruck, den Gültigkeitszeitraum und die SAN-Einträge. Das Status-Badge bedeutet:

- `Verfügbar` — ein gültiges Blattzertifikat ist geladen und wird verwendet;
- `Aktualisierung fehlgeschlagen` — der letzte Neuausstellungsversuch ist fehlgeschlagen; das vorherige Zertifikat bleibt aktiv und der Fehler wird auf der Seite angezeigt;
- `Nicht erzeugt` — weder CA noch Blattzertifikat existieren; mit `CA & Zertifikat erzeugen` beide erstellen.

`Blattzertifikat neu ausstellen` signiert sofort ein frisches Blattzertifikat, ohne auf einen automatischen Auslöser zu warten — nützlich direkt nach dem Ändern des Hostnamens oder der Schnittstellen, oder um SAN-Änderungen zu übernehmen, die außerhalb der Web-Oberfläche gemacht wurden.

## Verteilen der Root-CA

Clients, die die strikte Verifizierung nutzen sollen, benötigen das öffentliche Zertifikat der Root-CA. Es steht auf der Karte `Private Root-CA` bereit:

- `Root-CA herunterladen` speichert die PEM-Datei;
- `Root-CA kopieren` legt die PEM in der Zwischenablage ab, bereit zum Einfügen in das CA-Feld eines Clients.

Die CA ändert sich über Blattzertifikat-Erneuerungen und SAN-Bearbeitungen hinweg nie — einmal verteilen genügt. Bevor Sie die strikte Verifizierung irgendwo aktivieren, vergleichen Sie den auf dieser Seite angezeigten SHA-256-Fingerabdruck mit dem, was der Client erhalten hat (siehe die letzte Seite, Strikte Verifizierung auf Clients).

> Download und Zwischenablage enthalten nur das öffentliche CA-Zertifikat — niemals einen privaten Schlüssel. Geben Sie die CA nur an Personen weiter, die Ihren Server verifizieren können sollen.
