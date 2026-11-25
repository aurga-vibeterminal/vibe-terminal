Es gibt zwei Wege, WSL unter Windows zu nutzen: der leichte Weg fügt eine WSL-Shell als Terminal-Profil hinzu, der vollständige Weg installiert vterm-server innerhalb von WSL. Ersterer gibt Ihnen nur eine Shell; letzterer schaltet Dateiverwaltung, das Git-Panel, MCP und alles andere frei.

## Der leichte Weg: ein WSL-Terminal-Profil

Unter Windows **werden WSL-Terminalprofile automatisch geladen**: Beim Start erkennt der Server `wsl.exe` und zählt die installierten Distributionen auf, wobei er pro Distro einen Profileintrag in die Profilliste aufnimmt (z. B. `Ubuntu (WSL)`; ein generischer `WSL`-Eintrag erscheint, wenn keine Distros gelistet sind).

Zur Verwendung wählen Sie einfach den WSL-Eintrag im Profil-Menü der Titelleiste des Clients — die Distro startet und setzt Sie direkt in ihr Terminal ab, wie das Öffnen eines lokalen.

Einschränkung: Solche Sitzungen laufen auf dem Windows-seitigen Server — **Datei-Explorer, Git-Panel, Umgebungsvariablen und Co. des Clients funktionieren nicht** (sie arbeiten mit Windows-Dateisystem-Semantik und können das Linux-Dateisystem in WSL nicht sehen).

![Das Terminal-Profil-Menü — die automatisch gelisteten WSL-Distros](images/terminal-wsl-profile.jpg)

## Der vollständige Weg: vterm-server in WSL installieren

Behandeln Sie WSL als eigene Linux-Maschine und installieren Sie den Headless-vterm-server. Die Website bietet ein Einzeiler-Skript — fügen Sie es in ein WSL-Terminal ein:

```bash
curl -fsSL https://vterminal.dev/install.sh | bash
```

Das Skript automatisch:

1. Erkennt die WSL-/Headless-Umgebung und lädt den **Headless**-vterm-server nach `~/.local/vterm` herunter;
2. Installiert einen **systemd-User-Dienst** und startet ihn sofort — er läuft bei jeder Anmeldung (WSL-Start inklusive), kein manueller Start nötig;
3. Fügt `~/.local/vterm` zu Ihrem PATH hinzu.

> Hat WSL systemd deaktiviert, weist das Skript darauf hin, `[boot]` `systemd=true` in `/etc/wsl.conf` zu ergänzen, auf der Windows-Seite `wsl --shutdown` auszuführen und das Installationsprogramm erneut laufen zu lassen.

## Die Konfigurationsseite öffnen

In WSL:

```bash
vterm-server --open
```

Der Befehl gibt die Web-UI-Adresse (`http://localhost:<config-port>/`) aus und versucht, einen Browser zu starten (unter WSL über `wslview`, den Windows-Standardbrowser). Dank WSL2s Localhost-Forwarding funktioniert **das Öffnen derselben Adresse in einem Windows-Browser** genauso gut. Solange der Server läuft, trägt die ausgegebene URL ein einmaliges Anmelde-Token (`?ticket=...`, über einen lokalen Socket ausgestellt), sodass die Seite direkt angemeldet geöffnet wird.

## (Optional) Den WSL-Server ins Host-Relay teilen

Läuft auf der Windows-Maschine ebenfalls vterm-server (als Relay), kann sich der WSL-Server darunter einklinken, sodass der Client beide „Maschinen“ über eine lokale Verbindung sieht:

1. Klicken Sie in der Web UI des Windows-Servers auf den QR-Code, um den Share-String zu kopieren;
2. Öffnen Sie die Web UI des WSL-Servers (Adresse aus dem vorherigen Abschnitt), klicken Sie auf den Einfügen-Button neben dem Feld `Relay Address`, um Adresse und Schlüssel automatisch einzutragen, dann `Apply`;
3. Solange der Windows-vterm-server läuft, sieht der Client die Terminals, Dateien und das Git des WSL-Servers unter dem lokalen Relay.

> Unter WSL2s Standard-NAT-Netzwerk erreicht WSL den Windows-Host über die Adresse des Hosts aus WSL-Sicht (das Standard-Gateway — der Nameserver in `cat /etc/resolv.conf`); mit gespiegeltem Netzwerk funktioniert `localhost`. Details zum Relay siehe das Relay-Kapitel.
