Der Datei-Explorer bedient das serverseitige Dateisystem. Die Werkzeugleiste oben im linken Panel schaltet zwischen Recent / Explorer / Git / Debug / Search um; Explorer ist der Dateibrowser.

## Der Home-Button: zum cwd des Terminals springen

Klicken Sie in der Werkzeugleiste auf `Home` (Haus-Button):

- Wechselt zur Ansicht `Explorer`;
- Springt zum **Arbeitsverzeichnis des aktiven Terminals** (seinem cwd);
- Heftet dieses Verzeichnis als Baum an und leert das Suchfeld — Sie starten immer von „wo das Terminal ist, das sehen Sie“.

![Der Datei-Explorer im Normalmodus — Home, Breadcrumbs und die flache Liste](images/files-explorer.jpg)

## Baummodus und Suche

- **Als Baum anheften**: der Pin-Button am rechten Ende der Breadcrumbs schaltet zwischen flacher Liste und Baummodus. Im Baummodus klappen Ordner zu einem einklappbaren Verzeichnisbaum auf (der Home-Sprung heftet automatisch an);
- **Suche**: das Suchfeld oben filtert die aktuelle Liste beim Tippen nach Namen; `Search in Folder` aus dem Kontextmenü von Recent führt eine rekursive Suche über ein ganzes Verzeichnis aus.

![Der Datei-Explorer im Baummodus — die angeheftete Wurzel klapppt zu einem Verzeichnisbaum auf](images/files-explorer-tree.jpg)

## Grundoperationen

Rechtsklick auf eine Datei oder einen Ordner:

- **Erstellen**: `New Folder` / `New File`;
- **Bearbeiten**: `Rename`; `Copy` / `Cut` / `Paste` (verzeichnisübergreifend);
- **Löschen**: `Delete`, mit Bestätigungsdialog;
- **Terminal-Anbindung**: `Open in Terminal` öffnet einen neuen Terminal-Tab in diesem Verzeichnis; `cd to Directory` cd-t das aktuelle Terminal dorthin;
- **OS-Anbindung**: `Reveal in File Explorer` zeigt ein lokales Verzeichnis im System-Dateimanager;
- **Pfade**: `Copy Full Path` / `Copy Relative Path`;
- **Bereitstellen**: `Host with HTTP server` veröffentlicht das Verzeichnis als HTTP-Dateidienst (siehe Mit HTTP-Server freigeben); `Open in App` öffnet es mit einer auf dem Server erkannten Agent-App.

Jede Operation wirkt auf das serverseitige Dateisystem; Remote-Sitzungen verhalten sich exakt wie lokale.
