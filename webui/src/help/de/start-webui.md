Die Web UI ist die eigene Konfigurationsseite des Servers: Jede Serverfunktion ist als Bereich organisiert und in der linken Seitenleiste gelistet. Sie ist kein Terminal — die tägliche Arbeit läuft im Desktop-Client oder in `vtermc` (siehe Überblick). Diese Seite erklärt Aufbau und allgemeines Verhalten; die Funktionskapitel behandeln die Einstellungen jedes Bereichs im Detail.

## Aufbau

- **Seitenleiste** — die Bereichsliste (unten), das Hilfe-Symbol neben dem Servernamen und die App-Version am unteren Rand; auf schmalen Bildschirmen gleitet sie von links herein, und der Hamburger-Button der Kopfzeile blendet sie ein und aus;
- **Kopfzeile** — der Titel des aktuellen Bereichs sowie oben rechts der Sprachwähler und der Hell/Dunkel-Umschalter (auf schmalen Bildschirmen sitzt dort auch das Hilfe-Symbol);
- **Inhalt** — das Formular des gewählten Bereichs, zentriert und scrollbar.

## Bereiche

In der Reihenfolge der Seitenleiste:

- `Server` — Port-, TLS-Zertifikats- und Authentifizierungseinstellungen: der `Secret Key`, der `QR Code` für die Kopplung, die Relay-Adresse und der Schalter `Auto-start on Login` sind hier zu finden (siehe Client verbinden und Einrichtungsanleitung). Unter Linux erklärt der Auto-Start-Hinweis außerdem die automatische Anmeldung im Display-Manager, um den Desktop vor einer lokalen Anmeldung zu erreichen (siehe Server-Setup in Remote Desktop & Simulator);
- `Terminal` — Terminal-Analyse: Prompt-Erkennung, automatische Bestätigung und LLM-Klassifizierung (siehe Auto Confirm);
- `UART` — serielle Terminal-Profile auf Basis lokaler UART-Ports (siehe UART Seriell);
- `Addons` — installierte Addons: aktivieren, bearbeiten oder hochladen (LLM-Anbieter sind Addons — siehe LLM);
- `LLM` — weist jeder KI-Funktion einen LLM-Anbieter zu (siehe LLM);
- `Sprache` — Spracherkennungsanbieter, Modell- und Transkriptionseinstellungen (siehe Server-Setup in Spracheingabe);
- `Umgebung` — Umgebungsvariablen-Sammlungen in einem Gruppenbaum (siehe Umgebungsvariablen);
- `MCP` — der für Code-Agents bereitgestellte Terminal-MCP-Server (siehe Terminal MCP);
- `DDNS` — DDNS-Anbieter und Synchronisierungseinstellungen (siehe DDNS);
- `Portweiterleitung` — der Pool weitergeleiteter Ports und die laufenden TLS-Weiterleitungen (siehe Portweiterleitung);
- `Static HTTP` — über HTTP bereitgestellte Ordner und die aktiven Dateifreigaben (siehe Mit HTTP-Server freigeben);
- `Remotedesktop` — Aufnahme-, Eingabe- und Streameinstellungen für den Remotedesktop (siehe Server-Setup in Remote Desktop & Simulator);
- `LLM-Statistiken` — Token-Nutzung, Anfragezahlen und Leistungsmetriken über alle LLM-Funktionen.

## Speichern & Tipps

- Die meisten Bereiche speichern kurz nach einer Bearbeitung automatisch — ein Speicherindikator und etwaige Fehler erscheinen am unteren Rand des Bereichs;
- `MCP`, `DDNS` und `Remotedesktop` halten Änderungen zurück, bis `Änderungen speichern` gedrückt wird: Solange Änderungen ausstehen, erscheint neben den Schaltflächen der Hinweis `Ungespeicherte Änderungen`, und `Zurücksetzen` verwirft sie;
- Aus dem mobilen Client geöffnet, folgt die Web UI der Sprache und dem Design der App und blendet die eigenen Sprach- und Design-Steuerungen aus.
