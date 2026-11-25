Screenshots und das Terminal greifen in einem Ablauf ineinander: Ein einziger globaler Hotkey nimmt jeden Bereich des Desktops auf, Sie versehen ihn mit Anmerkungen und kopieren oder speichern ihn — dann fügen Sie das Bild aus der Zwischenablage direkt in ein Terminal ein, wo es als Dateipfad landet, den das laufende Programm (ein Code-Agent eingeschlossen) öffnen und verwenden kann. Beide Hälften leben im Desktop-Client.

## Einen Screenshot machen

- Drücken Sie `Shift`+`Alt`+`A` an beliebiger Stelle: Der Hotkey ist global, er feuert also unabhängig davon, welche App gerade den Fokus hat, solange der Desktop-Client läuft (auf macOS `Shift`+`Command`+`A` drücken);
- Jede Anzeige dimmt unter einer Aufnahme-Overlayer. Ziehen Sie, um den Auswahlbereich zu zeichnen — er darf sich über mehrere Anzeigen erstrecken —, dann ziehen Sie in ihm, um ihn zu verschieben, oder an seinen Griffen, um ihn zu verändern; ein Abzeichen über der Auswahl zeigt ihre Größe in Pixeln;
- `Esc` bricht jederzeit ab.

![Die Aufnahme-Overlayer — ein Auswahlbereich mit Größenabzeichen und Anmerkungs-Symbolleiste](images/terminal-images-capture.jpg)

## Annotieren

Die Symbolleiste unter der Auswahl trägt links die Anmerkungs-Werkzeuge (Rechteck und Ellipse), in der Mitte sechs Farben und rechts drei Aktionsschaltflächen — Abbrechen (`✕`), Speichern (Download-Pfeil) und Kopieren (`✓`).

- Wählen Sie ein Werkzeug und eine Farbe, dann ziehen Sie innerhalb der Auswahl, um zu zeichnen; Striche werden auf die Auswahl zugeschnitten;
- `Ctrl`+`Z` macht die letzte Anmerkung rückgängig, `Ctrl`+`R` stellt sie wieder her;
- Eine neue Fläche außerhalb der aktuellen Auswahl zu ziehen verwirft diese — Anmerkungen eingeschlossen — und beginnt eine neue.

## Kopieren oder speichern

- `Kopieren` (`✓`) legt die annotierte Aufnahme in die Zwischenablage — bereit, um in ein Terminal (unten), einen Chat oder jede andere App eingefügt zu werden;
- `Speichern` (Download-Pfeil) schreibt ein PNG über einen Speichern-Dialog; der Dateiname lautet standardmäßig `vterm-screenshot-<timestamp>.png`.

## Ein Bild ins Terminal einfügen

- Kopieren Sie ein Bild in die Zwischenablage — aus dem Screenshot-Werkzeug oben, einem Browser oder der eigenen Aufnahme des Betriebssystems;
- Rechtsklick aufs Terminal und `Paste Image` wählen; der Eintrag erscheint nur, wenn die Zwischenablage tatsächlich ein Bild enthält;
- Das Bild wird zum Server der Sitzung hochgeladen und dort als temporäre Datei gespeichert; sein in Anführungszeichen gesetzter Pfad wird danach genau wie getippt an der Cursorposition des Terminals eingefügt — in Bracketed-Paste-Marker gewickelt, wenn die laufende App sie unterstützt, sodass Shells, Editoren und Agenten ihn sauber empfangen;
- Für das Programm ist es schlicht Text: Ein Code-Agent wie Claude Code oder opencode öffnet das Bild über den Pfad, und gewöhnliche Shell-Befehle können es als Argument verwenden.

![Terminal-Rechtsklick — der Paste-Image-Eintrag](images/terminal-images-paste.jpg)

## Hinweise

- Der eingefügte Pfad liegt auf dem Server der Sitzung: Bei einem Remote- oder WSL-Server wird die temporäre Datei auf dieser Maschine erzeugt, genau dort, wo die im Terminal laufenden Programme sie erreichen;
- Kombiniert mit `Kontext an Agents` ist das der schnellste Weg, einem Agenten etwas auf Ihrem Bildschirm zu zeigen — aufnehmen, einfügen, und der Agent hält das Bild als Datei;
- Unter Linux funktioniert die Aufnahme unter X11; native Wayland-Sitzungen unterstützen weder den globalen Hotkey noch die interaktive Overlayer, und wenn bereits eine andere X11-App `Shift`+`Alt`+`A` belegt hat, lässt sich der Hotkey nicht registrieren.
