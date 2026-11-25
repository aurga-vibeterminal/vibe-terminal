`vtermc` is de commandoregel-client: hij verbindt met een server, maakt of koppelt terminalsessies en beheert de opgeslagen serverlijst — allemaal binnen een gewone terminal, via SSH of op een headless-machine. Hij deelt `~/.vibeterminal/client.json` met de desktopclient, dus beide zien dezelfde servers.

## Een terminal starten

Voer `vtermc` uit zonder modusvlaggen om een nieuwe terminal op de lokale server (standaard `wss://localhost:5153`) met de gedetecteerde shell te maken en naar de huidige terminal te streamen. Typ `exit` (of `Ctrl+D`) in de externe shell om de sessie te beëindigen.

Sessies leven op de server: als de verbinding wegvalt — netwerkverlies, venster sluiten, `vtermc` afbreken — blijft de sessie draaien en kan later met `vtermc -s` opnieuw worden gekoppeld.

- `vtermc -p <id>` — starten met een specifiek terminalprofiel (de profielenlijst staat in de sectie `Profiles` van de Web UI);
- `vtermc -p` — zonder profiel-id: de kiezer toont de profielen van de server interactief;
- `vtermc -w <dir>` — de werkmap van de nieuwe sessie instellen.

Als stdin geen terminal is (een pipe of bestand), schakelt `vtermc` naar batchmodus: stdin wordt naar de sessie gevoerd en de uitvoer stroomt tot de sessie sluit — handig vanuit scripts of agents:

```sh
echo "make test" | vtermc
```

## Verbinden met een server

- `--server wss://host:port` — verbind met een specifieke server in plaats van de lokale (IPv6-adressen tussen haakjes: `wss://[fd00::1]:5153`); de omgevingsvariabele `VTERM_SERVER` stelt dezelfde standaard in;
- De handshake wordt beantwoord met de geheime sleutel van de server: via `-a`/`-s` toegevoegde servers dragen hun sleutel in `client.json`; de lokale server valt terug op de omgevingsvariabele `TERMINAL_GUI_SECRET_KEY` (of een platte-tekstsleutel in `server.json`).

## Servers toevoegen

`-a` slaat een server op in `client.json` en sluit af — er wordt nooit een terminal geopend. Twee manieren:

- `vtermc -a` — interactieve prompts: `Server name`, `Server address` en `Secret key`. Het adres moet `wss://host:port` zijn (ongeldige invoer wordt opnieuw gevraagd; een leeg adres annuleert). Een lege naam valt terug op het adres;
- `vtermc -a "vtshare1:..."` — plak de versleutelde deeltekenreeks. Klik op de koppel-`QR Code` in de sectie `Server` van de Web UI om die naar het klembord te kopiëren; `vtermc` ontsleutelt hem en vult naam, adres en geheime sleutel automatisch in.

Details die de moeite waard zijn:

- een bestaand adres opnieuw toevoegen werkt die invoer ter plaatse bij (zijn id blijft) in plaats van te dupliceren;
- lokale adressen worden geweigerd — `Local` is altijd impliciet beschikbaar en heeft nooit een invoer nodig;
- de invoeren worden ook getoond door de Server Manager van de desktopclient.

## Servers en sessies kiezen

`vtermc -s` opent twee kiezders op volledig scherm (vereist een interactieve terminal):

1. **Serverlijst** — eerst `Local`, daarna elke beheerde server als `Name  (adres)`. Navigeer met de pijltjestoetsen of `j`/`k`, spring met een cijfer, `Enter` kiest, `q` annuleert;
2. **Terminallijst** — eerst `New Terminal`, daarna de live sessies van die server met werkmap, titel en bezittende server. Koppel aan een bestaande sessie, of maak een nieuwe (die daarna `-p` / `-w` eert zoals een gewone `vtermc`-run).

Koppelen onderbreekt de sessie nooit — hij blijft tussen koppelacties op de server draaien.

### Een server verwijderen

Ga in de serverlijst naar een beheerde server en druk op `d`: de regel wordt rood en de hint vraagt om bevestiging. Druk op `y` (of `Enter`) om de invoer uit `client.json` te verwijderen, elke andere toets breekt af. `Local` kan niet worden verwijderd.

## Omgevingsvariabelen

Een nieuwe sessie erft een vaste reeks uit je shell — `PATH`, `HOME`, `USER`, `SHELL`, `TERM`, `WT_PROFILE_ID`, `PSMODULEPATH`:

- `--env-inherit <patterns>` — kommagescheiden lijst (of de omgevingsvariabele `VTERM_ENV_INHERIT`); jokers `*` worden ondersteund;
- `-e VAR=VALUE` — extra variabelen injecteren (herhaalbaar);
- `--no-env` — de sessie zonder enige geërfde variabelen starten.

```sh
vtermc -w ~/project -e RUST_LOG=debug --env-inherit "PATH,HOME,TERM"
```

## Alle opties

```text
-p, --profile [<ID>]          Terminalprofiel kiezen (weglaten voor interactieve lijst)
-w, --working-dir <DIR>       Werkmap instellen
-s, --select                  Server en bestaande terminal interactief kiezen
-a, --add [<SHARE>]           Server toevoegen (vtshare1:-deeltekenreeks, of interactief)
    --server <URL>            WebSocket-URL van de server [env: VTERM_SERVER]
-e, --env <VAR=VALUE>         Extra omgevingsvariabelen
    --no-env                  Geen omgevingsvariabelen erven
    --env-inherit <PATTERNS>  Te erven omgevingsvariabelepatronen [env: VTERM_ENV_INHERIT]
```
