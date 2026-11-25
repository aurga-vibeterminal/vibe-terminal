`vtermc` ist der Kommandozeilen-Client: Er verbindet sich mit einem Server, erstellt oder hängt Terminal-Sitzungen an und verwaltet die gespeicherte Serverliste — alles innerhalb eines schlichten Terminals, über SSH oder auf einer Headless-Maschine. Er teilt sich `~/.vibeterminal/client.json` mit dem Desktop-Client, sodass beide dieselben Server sehen.

## Ein Terminal starten

Führen Sie `vtermc` ohne Modus-Flags aus, um ein neues Terminal auf dem lokalen Server (standardmäßig `wss://localhost:5153`) mit der erkannten Shell zu erstellen und in das aktuelle Terminal zu streamen. Geben Sie `exit` (oder `Ctrl+D`) in der Remote-Shell ein, um die Sitzung zu beenden.

Sitzungen leben auf dem Server: Bricht die Verbindung ab — Netzwerkverlust, geschlossenes Fenster, `vtermc` abgewürgt — läuft die Sitzung weiter und kann später mit `vtermc -s` wieder angehängt werden.

- `vtermc -p <id>` — mit einem bestimmten Terminal-Profil starten (die Profilliste steht im Bereich `Profiles` der Web UI);
- `vtermc -p` — ohne Profil-ID: die Auswahl listet die Profile des Servers interaktiv auf;
- `vtermc -w <dir>` — das Arbeitsverzeichnis der neuen Sitzung setzen.

Wenn stdin kein Terminal ist (Pipe oder Datei), wechselt `vtermc` in den Batch-Modus: stdin wird in die Sitzung gespeist und die Ausgabe streamt, bis die Sitzung schließt — praktisch aus Skripten oder Agents:

```sh
echo "make test" | vtermc
```

## Mit einem Server verbinden

- `--server wss://host:port` — statt mit dem lokalen mit einem bestimmten Server verbinden (IPv6-Adressen in Klammern: `wss://[fd00::1]:5153`); die Umgebungsvariable `VTERM_SERVER` setzt denselben Standard;
- Auf den Handshake wird mit dem Secret Key des Servers geantwortet: über `-a`/`-s` hinzugefügte Server tragen ihren Schlüssel in `client.json`; der lokale Server fällt auf die Umgebungsvariable `TERMINAL_GUI_SECRET_KEY` (oder einen Klartextschlüssel in `server.json`) zurück.

## Server hinzufügen

`-a` speichert einen Server in `client.json` und beendet sich — es öffnet nie ein Terminal. Zwei Wege:

- `vtermc -a` — interaktive Eingaben: `Server name`, `Server address` und `Secret key`. Die Adresse muss `wss://host:port` sein (ungültige Eingaben werden erneut abgefragt; eine leere Adresse bricht ab). Ein leerer Name fällt auf die Adresse zurück;
- `vtermc -a "vtshare1:..."` — den verschlüsselten Share-String einfügen. Ein Klick auf den Pairing-`QR Code` im Bereich `Server` der Web UI kopiert ihn in die Zwischenablage; `vtermc` entschlüsselt ihn und trägt Name, Adresse und Secret Key automatisch ein.

Details, die man wissen sollte:

- Erneutes Hinzufügen einer bestehenden Adresse aktualisiert den Eintrag an Ort und Stelle (seine ID bleibt) statt ihn zu duplizieren;
- Lokale Adressen werden abgelehnt — `Local` ist immer implizit verfügbar und braucht nie einen Eintrag;
- Die Einträge zeigt auch der Server-Manager des Desktop-Clients.

## Server und Sitzungen auswählen

`vtermc -s` öffnet zwei Vollbild-Auswahldialoge (erfordert ein interaktives Terminal):

1. **Serverliste** — zuerst `Local`, dann jeder verwaltete Server als `Name  (Adresse)`. Navigation mit Pfeiltasten oder `j`/`k`, Sprung mit einer Ziffer, `Enter` wählt, `q` bricht ab;
2. **Terminalliste** — zuerst `New Terminal`, danach die Live-Sitzungen dieses Servers mit Arbeitsverzeichnis, Titel und besitzendem Server. Hängen Sie eine bestehende Sitzung an oder erstellen Sie eine neue (die dann wie ein normaler `vtermc`-Aufruf `-p` / `-w` beachtet).

Anhängen unterbricht die Sitzung nie — sie läuft zwischen dem Anhängen auf dem Server weiter.

### Einen Server entfernen

Bewegen Sie sich in der Serverliste zu einem verwalteten Server und drücken Sie `d`: die Zeile wird rot und der Hinweis fordert zur Bestätigung auf. Drücken Sie `y` (oder `Enter`), um den Eintrag aus `client.json` zu löschen, jede andere Taste bricht ab. `Local` kann nicht gelöscht werden.

## Umgebungsvariablen

Eine neue Sitzung erbt eine feste Menge aus Ihrer Shell — `PATH`, `HOME`, `USER`, `SHELL`, `TERM`, `WT_PROFILE_ID`, `PSMODULEPATH`:

- `--env-inherit <patterns>` — kommagetrennte Liste (oder die Umgebungsvariable `VTERM_ENV_INHERIT`); `*`-Wildcards werden unterstützt;
- `-e VAR=VALUE` — zusätzliche Variablen injizieren (wiederholbar);
- `--no-env` — die Sitzung ganz ohne geerbte Variablen starten.

```sh
vtermc -w ~/project -e RUST_LOG=debug --env-inherit "PATH,HOME,TERM"
```

## Alle Optionen

```text
-p, --profile [<ID>]          Terminal-Profil wählen (weglassen für interaktive Liste)
-w, --working-dir <DIR>       Arbeitsverzeichnis setzen
-s, --select                  Server und bestehendes Terminal interaktiv wählen
-a, --add [<SHARE>]           Server hinzufügen (vtshare1:-Share-String oder interaktiv)
    --server <URL>            WebSocket-URL des Servers [env: VTERM_SERVER]
-e, --env <VAR=VALUE>         Zusätzliche Umgebungsvariablen
    --no-env                  Umgebungsvariablen nicht erben
    --env-inherit <PATTERNS>  Zu erbende Umgebungsvariablen-Muster [env: VTERM_ENV_INHERIT]
```
