Heim- und Büro-Breitband kommt typischerweise mit einer dynamischen öffentlichen IP, die sich ohne Ankündigung ändert. DDNS hält eine feste Domain auf die aktuelle IP der Maschine gerichtet — Clients und andere Server müssen sich nur den Namen merken, nie die Adresse.

## Wie es funktioniert

![DDNS-Architektur — der Manager erkennt periodisch die lokale IP, das Addon aktualisiert den DNS-Eintrag über die Provider-API, Clients verbinden sich per Domain](diagram/ddns)

Die ganze Pipeline läuft automatisch im Server-Hintergrund:

1. Der **DDNS-Manager** aktualisiert in einem festen Zyklus (standardmäßig alle 5 Minuten, anpassbar über die Umgebungsvariable `TERMINAL_SERVER_DDNS_POLL_INTERVAL_SECS`); das Speichern der Konfiguration stößt eine sofortige Aktualisierung an, und nichts läuft, solange der Dienst deaktiviert ist;
2. Die **lokale IP-Erkennung** liest zuerst die gewählte Netzwerkschnittstelle, fällt auf routenbasierte Erkennung zurück und filtert das Ergebnis nach der Einstellung `IP Family` (IPv4 / IPv6 / dual);
3. Das **DDNS-Addon** — ein sandgekapseltes JS-Skript — erhält die entschlüsselte Konfiguration und die aktuelle IP und ruft die API des DNS-Providers über HTTP auf. Das eingebaute Cloudflare-Addon beispielsweise sucht die Zone per Domain und schreibt die A-/AAAA-Einträge auf die aktuelle IP um und löscht veraltete Einträge, wenn eine Adressfamilie verschwindet;
4. **Clients** lösen die Domain zur neuesten IP auf und verbinden sich per Namen.

## Konfiguration

Öffnen Sie den Bereich `DDNS` der Web UI:

1. Schalten Sie `Enable` ein. Sie können den Dienst auch aus lassen, während Sie alles ausfüllen — ein deaktivierter Dienst speichert seine Konfiguration, läuft aber nie;
2. Wählen Sie unter `Provider` einen Anbieter; `Cloudflare DDNS` ist eingebaut;
3. Füllen Sie die Provider-Felder aus (Cloudflare-Beispiel):
   - `Email` — die E-Mail des Cloudflare-Kontos;
   - `API Token` — ein Token mit der Berechtigung Zone → DNS → Edit;
   - `Domain` — die Stammdomain der Zone, z. B. `example.com`;
   - `Subdomain` — der Eintragsname innerhalb der Zone, z. B. `sub` für `sub.example.com`; für den Zonenstamm leer lassen;
4. Wählen Sie unter `Network Interface & IP` die `Interface` und die `IP Family`. `Dual Stack` erfordert, dass die Schnittstelle sowohl eine IPv4- als auch eine IPv6-Adresse hat;
5. Klicken Sie `Save`, dann `Sync Now` zum Prüfen. `Sync Status` zeigt die zuletzt übertragene IP und die Sync-Zeit, bei Fehlern zusätzlich die Fehlermeldung des Addons.

## Sicherheit

Provider-Zugangsdaten (API-Token) werden vor dem Speichern verschlüsselt (`enc-v2:`) und nur im Speicher entschlüsselt, während das Addon läuft; Addons laufen in einer sandgekapselten JS-Laufzeit mit ausgehendem HTTP als einziger Fähigkeit.

## Provider erweitern

Provider sind Addons: ein JSON-Manifest (das die Konfigurationsfelder deklariert, die die Web UI in ein Formular verwandelt) plus ein JS-Skript mit `run()`. Das eingebaute `cloudflare-ddns` ist die Referenzimplementierung — eigene Provider lassen sich im Bereich `Addons` als Zip hochladen.

## Zusammenspiel mit anderen Funktionen

- Löst die Domain, kann der QR-Code im Bereich `Server` den `Connection Type` auf `DDNS Domain` schalten, sodass Clients eine Domain-Verbindung scannen;
- Andere Server können `wss://your-domain:port` in ihre `Relay Address` eintragen, um die Relay-Kette über die Domain zu fahren (siehe das Relay-Kapitel).
