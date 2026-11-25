Terminals sind das Herzstück von Vibe Terminal: Jeder Tab beherbergt eine oder mehrere Terminal-Sitzungen, kann in Bereiche geteilt oder als eigenes Fenster herausgezogen werden. Terminals erreichen den lokalen Server sowie Remote- und Shared-Server über ein Relay.

## Terminals erstellen

- Klicken Sie in der Titelleiste auf `+`, um einen Tab mit dem Standardprofil zu öffnen;
- Klicken Sie auf den Chevron neben `+` für das Profil-Menü: Terminal-Profile, gruppiert pro Server (lokal, verbunden, geteilt) — klappen Sie eine Gruppe auf und wählen Sie ein Profil, um ein Terminal auf diesem Server zu öffnen;
- Ziehen Sie einen lokalen Ordner auf die Titelleiste und lassen Sie ihn fallen, um einen neuen Tab mit diesem Arbeitsverzeichnis zu öffnen (nur lokaler Server; das Element muss ein Ordner sein).

![Titelleiste — der Neue-Tab-Button und das Profil-Menü](images/terminal-usage-creation.jpg)

## Maus-Erfassung und Kontextmenü

- Rechtsklick in einem Terminal öffnet sein Kontextmenü: Einfügen, Split, Quick Commands, Umgebungsvariablen-Sammlungen, Gesamttext kopieren, MCP-Einrichtungszeilen und mehr, je nach Sitzung;
- Anwendungen, die die Maus abfangen (opencode, pi, vim, tmux, htop, …), erhalten die Mausereignisse stattdessen — halten Sie `Shift` gedrückt, um die gewohnten Operationen auch bei solchen Apps auszuführen:
  - **Text kopieren**: Halten Sie `Shift` und wählen Sie den Text aus, klicken Sie dann — `Shift` weiter gedrückt — auf eine andere Stelle, um die Auswahl aufzuheben; die Auswahl wird in die Zwischenablage kopiert;
  - **Kontextmenü**: `Shift` + Rechtsklick oder `Ctrl` + Rechtsklick öffnet das Kontextmenü; ein Rechtsklick, während die linke Maustaste gehalten wird, öffnet es ebenfalls (eine vorhandene Auswahl wird vorher aufgehoben);
  - **Links**: `Shift` + Klick folgt Links wie gewohnt.

## Splits und Drag-Organisation

- **Split**: Rechtsklick im Terminal → `Split` → `Left` / `Right` / `Top` / `Bottom` teilt den Tab in Bereiche, jeder ein eigenständiges Terminal;

  ![Tab teilen — das Rechtsklick-Split-Menü und die entstehenden Bereiche](images/terminal-usage-split.gif)

- **Per Drag zusammenführen**: Ziehen Sie einen Tab auf ein anderes Terminal, und Docking-Zonen erscheinen (links/rechts/oben/unten); fallen lassen führt den Tab als neuen Bereich in den Ziel-Tab zusammen;

  ![Drag-Zusammenführung — ein auf ein anderes Terminal gezogener Tab wird ein neuer Bereich](images/terminal-usage-merge.mp4)

- **In ein Fenster lösen**: Ziehen Sie einen Tab aus der Tab-Leiste (oder auf ein anderes Fenster) und der ganze Tab verlässt sein Fenster als eigenes. Sitzungen laufen immer auf dem Server — Splits, Zusammenführungen und Drags unterbrechen niemals einen laufenden Befehl.

## Tabs und Tab-Gruppen

- **Tab** — ein Terminal-Tab, eventuell mit mehreren Split-Bereichen;
- **Tab-Gruppe** — ein Container, der Tabs organisiert. Standardmäßig folgen Gruppen den Servern: Die Tabs des lokalen Servers liegen in `Ungrouped`, und jeder Shared-Server erhält eine Gruppe `Name (Server)`; eigene Gruppen werden unterstützt;
- **Gruppen und Fenster** — jedes Fenster zeigt über den Gruppenwähler der Titelleiste eine aktive Gruppe (oder `All`); `Move Group to New Window` reißt eine ganze Gruppe in ein eigenes Fenster und widmet so ein Fenster oder einen Monitor einer Maschine;
- **Gruppen und Server** — eine Gruppe merkt sich ihren Server, sodass der Wechsel der Gruppe den Arbeitsbereich eines Servers refokussiert; das Profil-Menü folgt der aktiven Gruppe und zeigt nur die Profile relevanter Server.

![Der Gruppenwähler — Gruppenliste und In-neues-Fenster-verschieben](images/terminal-usage-groups.jpg)
