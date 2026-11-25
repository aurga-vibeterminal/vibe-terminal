Standardmäßig verschlüsselt ein Client jede Verbindung, prüft aber nicht, wer auf der anderen Seite ist — dieselbe Vertrauen-beim-ersten-Gebrauch-Haltung wie das Akzeptieren eines selbstsignierten Zertifikats im Browser. **Strikte Verifizierung** ändert das: Der Client prüft das Serverzertifikat gegen eine Root-CA Ihrer Wahl und verweigert die Verbindung, sofern Kette, Gültigkeit und SANs nicht alle in Ordnung sind. Das macht Man-in-the-Middle-Angriffe auf die Verbindung praktisch unmöglich — zum Preis eines manuellen Schritts: dem Client die Root-CA des Servers zu geben.

## Wo es eingerichtet wird

Die strikte Verifizierung wird pro Server im Server-Bearbeitungsbereich des Clients konfiguriert, neben Adresse und Geheimschlüssel:

1. `Root-CA-Zertifikat (PEM)` — ein mehrzeiliges Feld für das öffentliche Root-CA-Zertifikat des Servers;
2. `Strikte CA-Verifizierung (Kette, Gültigkeit & SAN)` — erst auswählbar, sobald eine CA eingetragen wurde.

![Der Server-Bearbeitungsbereich im Desktop-Client: mehrzeiliges Feld „Root-CA-Zertifikat (PEM)" mit dem Kontrollkästchen für die strikte CA-Verifizierung darunter](images/client-ca-fields.jpg)

## Die Root-CA zum Client bringen

Zwei Wege, beide aus derselben Quelle (die Seite `Zertifikate & TLS` des Servers — siehe die Seite Domains & SAN verwalten):

- **Kopieren & Einfügen** — in der Web-Oberfläche `Root-CA kopieren` drücken und die PEM in das Feld `Root-CA-Zertifikat (PEM)` des Clients einfügen;
- **Freigabe-String importieren** — vom Client erzeugte Freigabe-Strings betten die Root-CA zusammen mit Adresse und Schlüssel ein. Beim Importieren einer solchen Freigabe wird das CA-Feld automatisch vorbelegt; der Empfänger muss nur noch das Kontrollkästchen für die strikte Verifizierung aktivieren.

Die CA ist für jeden Client dieselbe und ändert sich über Zertifikatserneuerungen hinweg nie — einmal pro Server eintragen genügt.

## Was der strikte Modus prüft

Mit aktiviertem Kontrollkästchen muss jede Verbindung zu diesem Server ein Zertifikat vorweisen, das:

1. zur konfigurierten Root-CA führt — ein von etwas anderem signiertes Zertifikat wird abgelehnt;
2. innerhalb seines Gültigkeitszeitraums liegt — abgelaufene oder noch nicht gültige Zertifikate werden abgelehnt;
3. ein SAN trägt, das zur Adresse passt, zu der Sie sich verbinden — ein für andere Namen ausgestelltes Zertifikat wird abgelehnt.

Schlägt eine der Prüfungen fehl, wird die Verbindung mit einem Fehler verweigert, der angibt, welche Prüfung fehlgeschlagen ist. Ohne das Kontrollkästchen (oder ohne CA) bleiben Verbindungen verschlüsselt, aber diese Prüfungen werden übersprungen.

## Den Fingerabdruck verifizieren

Bevor Sie der CA vertrauen, bestätigen Sie, dass Sie die echte haben: Die Seite `Zertifikate & TLS` zeigt den SHA-256-Fingerabdruck der Root-CA. Vergleichen Sie ihn mit der PEM, die Sie gerade einfügen wollen (oder der PEM, die in einer erhaltenen Freigabe eingebettet ist). Weichen sie ab, aktivieren Sie die strikte Verifizierung nicht — besorgen Sie die CA stattdessen über einen vertrauenswürdigen Kanal.

## Häufige Fehlerszenarien

- `SAN stimmt nicht überein` — Sie verbinden sich über einen Namen oder eine IP, die nicht im Zertifikat steht. Fügen Sie ihn auf dem Server hinzu (siehe die Seite Domains & SAN verwalten) und stellen Sie neu aus;
- `Zertifikat abgelaufen oder noch nicht gültig` — die Uhr des Servers geht falsch, oder das Blattzertifikat konnte nicht erneuert werden. Prüfen Sie den Zertifikatsstatus in der Web-Oberfläche;
- `Nicht von der konfigurierten CA signiert` — die CA im Feld gehört zu einem anderen Server, oder die Verbindung wird abgefangen. Vergleichen Sie Fingerabdrücke, bevor Sie fortfahren.
