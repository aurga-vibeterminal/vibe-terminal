Die Dateiverwaltung lässt Sie vom Client aus direkt mit Dateien auf dem Server arbeiten: Die linke Aktivitätsleiste hat zwei Einträge — `Recent` (zuletzt geöffnete Projekte) und den Datei-Explorer —, die neben Terminal-Tabs genutzt werden. Alles arbeitet auf dem Server der Sitzung: die lokale Maschine oder eine Remote-/Shared-Maschine über ein Relay.

## Recent: zuletzt geöffnete Projekte

`Recent` listet die Ordner, die Ihre Code-Agents auf dem aktuellen Server zuletzt geöffnet haben, neueste zuerst:

- **Klick** auf einen Eintrag wechselt den Datei-Explorer in dieses Verzeichnis;
- **Rechtsklick** öffnet ein Menü schneller Aktionen:
  - `Open in VS Code` — das Verzeichnis in VS Code auf dieser Maschine öffnen;
  - `Reveal in File Explorer` — es im Dateimanager des Betriebssystems zeigen;
  - `Copy Full Path` / `Copy Relative Path` — vollständiger Pfad oder zum Repository relativer Pfad;
  - `Search in Folder` — Dateien in diesem Verzeichnis durchsuchen;
  - `Open in Terminal` — dort einen neuen Terminal-Tab öffnen; `cd Here` — das aktuelle Terminal dorthin cd-en;
  - `Host with HTTP server` — das Verzeichnis als HTTP-Dateidienst veröffentlichen (siehe Mit HTTP-Server freigeben);
  - `Open in App` — es mit einer auf dem Server erkannten Agent-App öffnen.

![Die linke Leiste — die Liste der zuletzt geöffneten Projekte](images/files-overview.jpg)

## Der Datei-Explorer

Der Tab `Explorer` durchstöbert und verwaltet das serverseitige Dateisystem: Verzeichnisnavigation, Baum- und Flachtansicht, Suche, Erstellen/Umbenennen/Löschen und weitere Dateioperationen, dazu Drag-and-Drop-Upload und -Download zwischen dem Server und dieser Maschine. Siehe Datei-Explorer und Hoch- & Herunterladen.
