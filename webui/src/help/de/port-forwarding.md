Die Portweiterleitung veröffentlicht einen HTTP-Dienst, der auf dem Server läuft — einen Dev-Server, ein Geräte-Dashboard, alles, was auf einer einfachen `http://localhost:port`-Adresse lauscht — auf einem eigenen TLS-Port, sodass jedes Gerät, das den Server erreichen kann, ihn im Browser als `https://<server-adresse>:<port>/` öffnet.

## Wie es funktioniert

- Eine Weiterleitung bildet einen **Zielport** — den HTTP-Dienst auf dieser Maschine — auf einen **weitergeleiteten Port** ab, der aus dem konfigurierten Portbereich (Standard `62000`–`62999`) zugeteilt wird;
- Der weitergeleitete Port spricht TLS: Der gesamte Datenverkehr wird mit dem **selbstsignierten Zertifikat** des Servers verschlüsselt — nichts läuft im Klartext, selbst über ein nicht vertrauenswürdiges Netzwerk;
- Jedes Gerät, das den Server erreichen kann — im LAN oder aus dem Internet, sobald der Portbereich am Router weitergeleitet ist — öffnet `https://<server-adresse>:<forwardedPort>/` und landet auf dem lokalen Dienst.

Weiterleitungen werden bei Bedarf erstellt: Die Dateifreigabe `Host with HTTP server` auf einem Remote-Server (über ein Relay verbunden) fordert automatisch eine an und übergibt Ihnen die fertig zu öffnende URL. Keine manuelle Einrichtung nötig.

## Weiterleitungen verwalten

![Web UI — der Bereich Portweiterleitung: Portbereichskonfiguration und die Liste laufender Weiterleitungen](images/port-forwarding-webui.jpg)

1. Öffnen Sie den Bereich `Portweiterleitung` der Web UI des Servers.
2. Die `Portbereichskonfiguration` begrenzt den Pool, aus dem weitergeleitete Ports zugeteilt werden; Änderungen werden automatisch gespeichert. Verkleinern Sie ihn, wenn Ihre Firewall oder Ihr Router nur wenige Ports weiterleitet.
3. Die Live-Liste zeigt jede Weiterleitung als `:forwardedPort → :targetPort` mit der Anzahl ihrer aktiven Verbindungen.
4. Der Stopp-Button einer Weiterleitung fährt den Listener sofort herunter und trennt ihre aktiven Verbindungen; die nächste Freigabe bekommt einen frischen Port zugeteilt. `Aktualisieren` liest die Liste neu ein.

## Die Zertifikatswarnung

Weiterleitungen werden mit dem eigenen selbstsignierten Zertifikat des Servers verschlüsselt — demselben, das auch die Web UI und Client-Verbindungen schützt. Browser können es nicht gegen eine Zertifizierungsstelle prüfen, daher zeigt der erste Besuch einer weitergeleiteten URL eine Datenschutzwarnung (*Ihre Verbindung ist nicht privat*). Das ist erwartet und kann gefahrlos weggeklickt werden: Die Verschlüsselung selbst ist nicht kompromittiert, nur die Herkunft des Zertifikats lässt sich nicht maschinell verifizieren. Um die Warnung dauerhaft abzustellen, importieren Sie das Zertifikat des Servers in den Vertrauensspeicher Ihres Geräts.

![Browser — die Warnung vor dem nicht vertrauenswürdigen selbstsignierten Zertifikat; über Erweitert → Weiter die Seite öffnen](images/port-forwarding-cert-warning.jpg)

## Chrome unter macOS zeigt `ERR_ADDRESS_UNREACHABLE`

Ab macOS 15 (Sequoia) benötigen Drittanbieter-Apps eine explizite Erlaubnis, um Geräte im lokalen Netzwerk zu erreichen — jede `192.168.x.x`-, `10.x.x.x`- oder ähnliche Adresse. Safari ist als System-App ausgenommen, sodass sich eine weitergeleitete URL in Safari problemlos öffnen lässt, während Chrome `ERR_ADDRESS_UNREACHABLE` meldet, bevor TLS überhaupt beginnt. Das ist weder ein Zertifikatsproblem noch ein Problem mit der Weiterleitung selbst.

So erteilen Sie Chrome den Zugriff:

1. Öffnen Sie `Systemeinstellungen` → `Datenschutz & Sicherheit` → `Lokales Netzwerk`.
2. Suchen Sie `Google Chrome` in der Liste und schalten Sie es **ein**.
3. Beenden Sie Chrome vollständig (`Cmd+Q`, nicht nur Fenster schließen) und starten Sie es neu.

Ist der Schalter bereits eingeschaltet, schalten Sie ihn aus und wieder ein — er kann nach einem Chrome- oder macOS-Update hängen bleiben. Bleibt der Fehler bestehen, prüfen Sie, ob eine VPN- oder Proxy-Erweiterung in Chrome private Adressbereiche nicht weiterleitet.

## Zugangskontrolle

Was einen weitergeleiteten Port schützt, hängt davon ab, was dahinterliegt:

- **Eingebaute Freigaben sind tokengeschützt.** Die Freigabelinks, die der Client erzeugt — eine `Host with HTTP server`-Freigabe (`?share_token=…`) und die Settings-Seite (`?ticket=…`) — enthalten ein Zugriffstoken. Eine Verbindung ohne gültiges Token und ohne gültige Sitzung wird getrennt, bevor irgendetwas ausgeliefert wird. Beim ersten Besuch tauscht der Browser das Token gegen ein sicheres Sitzungs-Cookie ein, das nur für diesen einen Dienst gilt, und das Token wird aus der URL entfernt — es bleibt weder in der Adressleiste noch im Verlauf.

- **Die Tokens sind einmalig und kurzlebig.** Ein Settings-Ticket funktioniert genau einmal und läuft nach einer Minute ab; sobald eine Freigabe gestoppt oder neu gestartet wird, funktionieren frühere Links und Sitzungen nicht mehr. Behandeln Sie den vollständigen Link als Berechtigungsnachweis — wer ihn hat, kann auf den Dienst zugreifen — und teilen Sie ihn nur mit Menschen, denen Sie vertrauen.

- **Eigene Dienste bringen ihre eigene Authentifizierung mit.** Eine Weiterleitung zu einem anderen lokalen Dienst verschlüsselt den Datenverkehr mit dem selbstsignierten Zertifikat des Servers, fügt aber keine eigene Authentifizierung hinzu: Alles, was der Dienst ausliefert — Seiten wie APIs —, erreicht jeder, der den Port erreichen kann. Schützen Sie sensible Dienste mit einer Authentifizierung, bevor Sie sie freigeben.
