## Der Browser warnt „Ihre Verbindung ist nicht privat“ — ist das normal?

Ja. Weitergeleitete Ports, die Web UI und Client-Verbindungen teilen sich dasselbe selbstsignierte Serverzertifikat; die Verschlüsselung selbst ist nicht kompromittiert. Importieren Sie das Serverzertifikat in Ihren Vertrauensspeicher, um die Warnung dauerhaft abzustellen.

## Ist Vibe Terminal kostenlos?

Ja — der Desktop-Client/Server und der Headless-Server sind kostenlos. Laden Sie sie herunter und installieren Sie sie auf so vielen Rechnern, wie Sie möchten.

## Wohin gehen meine Daten?

Nirgendwohin, wo Sie sie nicht selbst hinlegen. Sitzungen, Zugangsdaten (enc-v2:) und Spracherkennung leben ausschließlich auf Ihren Rechnern; Kopplungsschlüssel reisen nie im Klartext.

## Kann ich einen weitergeleiteten Dienst sicher freigeben?

Ja. Die eingebauten Freigaben sind tokengeschützt: Die Links, die der Client für eine `Host with HTTP server`-Freigabe oder die Settings-Seite erzeugt, enthalten ein Zugriffstoken; eine Verbindung ohne gültiges Token und ohne gültige Sitzung wird getrennt, bevor irgendetwas ausgeliefert wird. Beim ersten Besuch wird das Token gegen ein sicheres Sitzungs-Cookie eingetauscht, das nur für diesen einen Dienst gilt, und verschwindet aus der URL — ein Settings-Ticket funktioniert genau einmal und läuft nach einer Minute ab, und sobald eine Freigabe gestoppt oder neu gestartet wird, sind frühere Links und Sitzungen ungültig. Behandeln Sie den vollständigen Link als Berechtigungsnachweis und geben Sie ihn nur an Menschen weiter, denen Sie vertrauen. Eine Weiterleitung zu Ihrem eigenen Dienst ergänzt TLS mit dem Serverzertifikat, aber keine Authentifizierung — halten Sie Authentifizierung vor sensiblen Diensten und reißen Sie eine Weiterleitung über das Portweiterleitungs-Panel sofort ab, wenn Sie fertig sind.
