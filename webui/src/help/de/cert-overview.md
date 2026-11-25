Der Server schützt alle TLS-Endpunkte, die er anbietet — die Web-Oberfläche, Client-Verbindungen und weitergeleitete Ports — mit einem Zertifikat aus seiner eigenen **privaten Root-CA**. Die Seite `Zertifikate & TLS` in der Web-Oberfläche ist der Ort, an dem Sie diese CA verwalten, festlegen, für welche Namen das Zertifikat gültig ist, und die CA an Clients weitergeben, die den Server strikt verifizieren wollen.

## Funktionsweise

- Der Server erzeugt eine private Root-CA genau einmal und behält sie für die Lebensdauer der Installation. Alles wird von ihr signiert;
- Die CA signiert ein **Blattzertifikat** für den Server mit einer Gültigkeit von 90 Tagen, das die Hostnamen und IP-Adressen abdeckt, die Clients tatsächlich zum Erreichen des Servers verwenden;
- Ändern sich Hostnamen, Netzwerkschnittstellen oder SAN-Einträge der Maschine, wird das Blattzertifikat automatisch neu ausgestellt — die Root-CA ändert sich nie, daher vertrauen Clients, die ihr bereits vertrauen, dem Server weiterhin;
- Ein neues Blattzertifikat wird über einen gemeinsamen Zertifikats-Resolver eingetauscht und ohne Neustart geladen: bestehende Verbindungen werden nie getrennt, und neue Handshakes verwenden sofort das neueste Zertifikat. Schlägt eine Erneuerung fehl, bleibt das vorherige Zertifikat aktiv.

![Die Seite „Zertifikate & TLS": oben die Karte „Private Root-CA", in der Mitte die SAN-Verwaltung, unten die Details des Server-Blattzertifikats](images/certificates-page.jpg)

## Was Sie auf der Seite finden

1. `Private Root-CA` — das öffentliche CA-Zertifikat herunterladen oder kopieren, um es an Clients (und deren Browser) zu verteilen. Der Download enthält niemals einen privaten Schlüssel;
2. `Domains & IP-Adressen (SAN)` — die Liste der Namen, für die das Blattzertifikat gültig ist. Beschrieben auf der nächsten Seite, Domains & SAN verwalten;
3. `Server-Blattzertifikat` — das aktive Zertifikat: Status, Antragsteller, SHA-256-Fingerabdruck, Gültigkeitszeitraum und SAN-Einträge.

## Benutzerdefinierte TLS-Zertifikate

Ist für den Server ein benutzerdefiniertes TLS-Zertifikat konfiguriert, bleibt es in der Verantwortung: das verwaltete Zertifikat auf dieser Seite wird weiterhin erzeugt und aktuell gehalten, ersetzt aber nicht die konfigurierte benutzerdefinierte Identität. Solange eine benutzerdefinierte Identität aktiv ist, zeigt die Seite einen Hinweis an.

## Warum der Root-CA vertrauen?

Clients, die nur Verschlüsselung benötigen, funktionieren ohne weitere Einrichtung. Clients, die den Server *verifizieren* wollen — der strikte Modus, beschrieben auf der letzten Seite, Strikte Verifizierung auf Clients — benötigen das öffentliche Zertifikat der Root-CA, und es muss über einen Kanal übertragen werden, dem Sie vertrauen:

> Laden Sie die Root-CA nur über die lokale Web-Oberfläche dieses Servers oder über einen vertrauenswürdigen Tunnel herunter, und prüfen Sie den auf dieser Seite angezeigten SHA-256-Fingerabdruck, bevor Sie den strikten Modus irgendwo aktivieren.

Derselben CA vertrauen auch die Zertifikate der Portweiterleitung (siehe die Seite Portweiterleitung). Sobald ein Gerät der Root-CA vertraut, öffnen sich weitergeleitete URLs ohne die Browser-Warnung *Verbindung nicht privat*.
