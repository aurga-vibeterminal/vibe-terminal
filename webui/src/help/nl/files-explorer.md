De bestandsverkenner bedient het bestandssysteem aan serverzijde. De werkbalk bovenaan het linkerpaneel schakelt tussen Recent / Explorer / Git / Debug / Search; Explorer is de bestandsbrowser.

## De Home-knop: springen naar de cwd van de terminal

Klik op de knop `Home` (huisje) in de werkbalk:

- Schakelt naar de weergave `Explorer`;
- Springt naar de **werkmap van de actieve terminal** (zijn cwd);
- Pint die map vast als boom en wist het zoekvak — je start altijd vanaf "waar de terminal is, dat zie je".

![De bestandsverkenner in normale modus — Home, kruimelspoor en de vlakke lijst](images/files-explorer.jpg)

## Boommodus en zoeken

- **Vastpinnen als boom**: de pinknop rechts in het kruimelspoor wisselt tussen de vlakke lijst en de boommodus. In boommodus vouwen mappen uit tot een inklapbare mappenboom (de Home-sprong pint automatisch vast);
- **Zoeken**: het zoekvak bovenin filtert de huidige lijst op naam terwijl je typt; `Search in Folder` uit het contextmenu van Recent voert een recursieve zoekopdracht over een hele map uit.

![De bestandsverkenner in boommodus — de vastgepinde wortel vouwt uit tot een inklapbare mappenboom](images/files-explorer-tree.jpg)

## Basisbewerkingen

Rechtsklik op een bestand of map:

- **Aanmaken**: `New Folder` / `New File`;
- **Bewerken**: `Rename`; `Copy` / `Cut` / `Paste` (werkt over mappen heen);
- **Verwijderen**: `Delete`, met een bevestigingsvenster;
- **Terminal-koppeling**: `Open in Terminal` opent een nieuw terminaltabblad in die map; `cd to Directory` cd't de huidige terminal erheen;
- **OS-koppeling**: `Reveal in File Explorer` toont een lokale map in de systeembestandsbeheerder;
- **Paden**: `Copy Full Path` / `Copy Relative Path`;
- **Publiceren**: `Host with HTTP server` publiceert de map als HTTP-bestandsdienst (zie Publiceren met HTTP-server); `Open in App` opent hem met een op de server gedetecteerde agent-app.

Elke bewerking werkt op het bestandssysteem aan serverzijde; externe sessies gedragen zich exact zoals lokale.
