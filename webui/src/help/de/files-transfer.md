Dateiübertragungen zwischen Ihrer Maschine und dem Server sind Drag & Drop, und die Richtung ist die Bedeutung: **hineinziehen = zum Server hochladen, herausziehen = auf diese Maschine herunterladen**. Übertragungen laufen über die verschlüsselte Client-Server-Verbindung; Remote- und Shared-Server verhalten sich wie lokale.

## Hochladen: hineinziehen

Ziehen Sie Dateien (oder ganze Ordner) aus dem Dateimanager Ihres Betriebssystems auf die Dateiliste und lassen Sie los:

- Im Baummodus lädt **Ablegen auf einer Ordnerzeile** in diesen Ordner hoch; **Ablegen an der Wurzel** (die Einfügelinie zwischen den Zeilen) lädt in das aktuelle Wurzelverzeichnis hoch;
- Unten in der Liste erscheint eine Übertragungszeile mit `Uploading`, dem Dateinamen und einem Fortschrittsbalken;
- Beim Abschluss erscheint `Uploaded n file(s)`; Fehlschläge enthalten den Grund (z. B. Zugriff verweigert) und die Liste aktualisiert sich und zeigt die neuen Dateien.

## Herunterladen: herausziehen

Drücken Sie eine Datei in der Liste, ziehen Sie sie aus dem Client-Fenster heraus und lassen Sie sie im Dateimanager Ihres Betriebssystems (oder auf dem Desktop) fallen:

- Die Datei streamt vom Server dorthin, wo Sie sie abgelegt haben;
- Übertragungszeilen zeigen den Fortschritt ebenso; **eine abgeschlossene Download-Zeile ist klickbar** und öffnet den lokalen Download-Ordner;
- Mehrfachauswahl und mehrere Dateien auf einmal ziehen.

![Übertragungsfortschritt — die Upload-/Download-Zeilen](images/files-transfer.jpg)

> Drag & Drop bewegt einzelne Dateien zwischen dieser Maschine und einem Server; um Daten zwischen zwei Servern zu bewegen, kombinieren Sie Mit HTTP-Server freigeben oder Terminal-Befehle (scp/rsync).
