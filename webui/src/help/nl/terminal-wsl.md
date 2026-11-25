Er zijn twee manieren om WSL onder Windows te gebruiken: de lichte manier voegt een WSL-shell toe als terminalprofiel, de volledige manier installeert vterm-server binnen WSL. De eerste geeft je alleen een shell; de tweede ontgrendelt bestandsbeheer, het Git-paneel, MCP en al het andere.

## De lichte manier: een WSL-terminalprofiel

Onder Windows **worden WSL-terminalprofielen automatisch geladen**: bij het opstarten detecteert de server `wsl.exe` en somt de geïnstalleerde distributies op, waarbij per distro één profielinvoer aan de lijst wordt toegevoegd (bijv. `Ubuntu (WSL)`; een algemene invoer `WSL` verschijnt als er geen distro's zijn vermeld).

Voor gebruik kies je simpelweg de WSL-invoer in het profielmenu van de titelbalk van de client — de distro start en zet je direct in de terminal ervan, zoals het openen van een lokale.

Beperking: zulke sessies draaien op de server aan de Windowskant — de **bestandsverkenner, het Git-paneel, omgevingsvariabelen en dergelijke van de client werken niet** (ze opereren met Windows-besturingssysteemsemantiek en kunnen het Linux-bestandssysteem binnen WSL niet zien).

![Het terminalprofielmenu — de automatisch vermelde WSL-distro's](images/terminal-wsl-profile.jpg)

## De volledige manier: vterm-server binnen WSL installeren

Behandel WSL als een eigen Linux-machine en installeer de headless vterm-server. De website biedt een one-liner-script — plak het in een WSL-terminal:

```bash
curl -fsSL https://vterminal.dev/install.sh | bash
```

Het script automatisch:

1. Detecteert de WSL/headless-omgeving en downloadt de **headless** vterm-server naar `~/.local/vterm`;
2. Installeert een **systemd-gebruikersdienst** en start die direct — hij draait bij elke login (inclusief WSL-start), geen handmatige start nodig;
3. Voegt `~/.local/vterm` toe aan je PATH.

> Als systemd in WSL is uitgeschakeld, wijst het script op het toevoegen van `[boot]` `systemd=true` aan `/etc/wsl.conf`, het uitvoeren van `wsl --shutdown` aan de Windowskant en het opnieuw draaien van het installatieprogramma.

## De configuratiepagina openen

Binnen WSL:

```bash
vterm-server --open
```

De opdracht print het Web UI-adres (`http://localhost:<config-port>/`) en probeert een browser te openen (onder WSL via `wslview`, de standaardbrowser van Windows). Dankzij WSL2's localhost-forwarding werkt **hetzelfde adres in een Windows-browser openen** net zo goed. Zolang de server draait, draagt de geprinte URL een eenmalig inlogtoken (`?ticket=...`, verstrekt via een lokale socket), zodat de pagina al aangemeld opent.

## (Optioneel) De WSL-server delen in de host-relay

Als de Windowsmachine ook vterm-server draait (als relay), kan de WSL-server eronder aansluiten zodat de client beide "machines" via één lokale verbinding ziet:

1. Klik in de Web UI van de Windowsserver op de QR-code om de deeltekenreeks te kopiëren;
2. Open de Web UI van de WSL-server (adres uit de vorige sectie), klik op de plakknop die naast het veld `Relay Address` verschijnt om adres en sleutel automatisch in te vullen, daarna `Apply`;
3. Zolang de Windows-vterm-server draait, ziet de client de terminals, bestanden en Git van de WSL-server onder de lokale relay.

> Onder WSL2's standaard NAT-netwerkbereikt WSL de Windows-host via het hostadres gezien vanaf WSL (de standaardgateway — de nameserver in `cat /etc/resolv.conf`); met gespiegeld netwerken werkt `localhost`. Zie het Relay-hoofdstuk voor relaydetails.
