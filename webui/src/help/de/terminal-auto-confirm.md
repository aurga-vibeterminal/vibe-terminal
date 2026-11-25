Der serverseitige Terminal-Analysator beobachtet die Terminal-Ausgabe: Er erkennt Befehls-Prompts, erkennt Aufgabenabschluss und klassifiziert mit einem LLM, was das Terminal zeigt. `Auto Confirm` ist die meistgenutzte Funktion — hält das Terminal an einem gewöhnlichen Bestätigungs-Prompt an (wie `Proceed? [Y/n]`), wird Enter für Sie gesendet, sodass lange Aufgaben nicht mehr an Fragen hängen bleiben.

## Konfiguration

Der Bereich `Terminal Analysis` der Web UI:

- `Terminal observer` — der Hauptschalter der serverseitigen Analyse;
- `Auto confirm` — sendet Enter nur für Prompts, die als **gewöhnliche Bestätigungen** klassifiziert sind; andere Fragen werden niemals blind beantwortet;
- `Analysis delay (seconds)` — wie lange nach dem Leerlauf eines Agents gewartet wird, bevor analysiert wird; vermeidet überflüssige LLM-Aufrufe zwischen Aufgaben;
- `Confirmation grace (seconds)` — das Fenster, in dem Sie nach erkannter Bestätigung übernehmen können;
- `Notify on completion` — gibt ein Ereignis aus, wenn Ausgabe als „Aufgabe abgeschlossen“ klassifiziert wird (für Benachrichtigungen nutzbar).

> Der Hinweis auf der Seite: Wenn der Auto-Modus Ihres eigenen Agents aktiv ist, empfiehlt es sich, `Auto confirm` im Client auszuschalten, damit die beiden Automatisierungen sich nicht gegenseitig überholen.

## LLM-Abhängigkeit

Die Klassifizierung übernimmt ein LLM: Wählen Sie im Bereich `LLM` einen `Provider` und tragen Sie seine Zugangsdaten ein (geteilt von Terminal-Klassifizierung, Git-Commit-Meldungen, dem Chat-Assistenten und weiteren KI-Funktionen). Ohne LLM arbeitet der Analysator weiterhin mit verringerter Genauigkeit — halten Sie Auto Confirm in dem Fall aus.

![Die Einstellungsseite Terminal Analysis](images/terminal-auto-confirm.jpg)

![Die LLM-Einstellungsseite — Provider wählen und Zugangsdaten eintragen](images/terminal-auto-confirm-llm.jpg)
