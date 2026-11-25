UART Serial verwandelt die serielle Konsole eines Geräts (Debug-Port, Embedded-Board, SBC) in einen Terminal-Tab, den Sie wie jedes andere Terminal beobachten und bedienen können.

## Einen Port konfigurieren

In der Web UI im Bereich `UART` → `Add UART Port`:

- `Name` — wird als Titel des Terminals angezeigt (z. B. „Debug Console“);
- `Serial Port` — aus den erkannten Ports wählen (die Anzahl steht oben auf der Seite);
- `Baud Rate` / `Data Bits` / `Parity` / `Stop Bits` / `Flow Control` — auf die Geräte-Firmware abstimmen;
- `Terminal Type` und `Encoding` — die Kodierung steht standardmäßig auf Auto (UTF-8) und kann der Firmware folgen;
- `Show Advanced Options` legt weitere serielle Parameter frei.

Gespeicherte Ports erscheinen in der Liste `Configured UART Ports` und können jederzeit bearbeitet oder entfernt werden.

![Die UART-Einstellungsseite — das Port-Formular](images/terminal-uart-config.jpg)

## Verwendung im Client

Konfigurierte Ports öffnen sich wie gewöhnliche Serversitzungen als Terminal-Tabs:

- Serielle Daten werden bei Eintreffen angezeigt; Tastatureingaben gehen direkt an den Port;
- Das Rechtsklick-Menü bietet `Clear` (Bildschirm leeren) und ein `Encoding`-Untermenü (Kodierung live wechseln; die aktuelle ist abgehakt);
- UART-Sitzungen haben keine Shell-Funktionen (keine Quick Commands, Apps oder Envs-Menüpunkte).

Typische Einsatzfälle: Router-/Switch-Konsolen, Protokolle von Embedded-Boards, Flash-Jig-Überwachung.
