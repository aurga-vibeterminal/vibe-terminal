Die Intelligenz des Git-Panels ist LLM-getrieben: Commit-Meldungen mit einem Klick, KI-gestütztes Diff-Lesen und agentenbasierte Konfliktlösung Hunk für Hunk. Alles beruht auf einem LLM-Provider, der in der Web UI konfiguriert wird.

## Den Provider konfigurieren

Öffnen Sie den Bereich `LLM` der Web UI:

- Wählen Sie einen `Provider` und tragen Sie seine Zugangsdaten ein (die Felder sind vom Manifest des Providers definiert); die Konfiguration ist global — Terminal-Klassifizierung, Git-Commit-Meldungen, der Chat-Assistent und weitere KI-Funktionen teilen sich denselben Provider;
- Provider sind Addons vom Typ LLM: gängige sind eingebaut, eigene Provider lassen sich im Bereich `Addons` als Zip hochladen (das Manifest deklariert die Konfigurationsfelder, die die Web UI in ein Formular verwandelt).

![Der LLM-Bereich der Web UI — Provider wählen und Zugangsdaten eintragen](images/git-llm-provider-config.jpg)

## LLM-Funktionen in Git

- **Commit-Meldung generieren** — klicken Sie auf den KI-Button neben der Commit-Box; das LLM liest das gestagte Diff und schreibt eine Meldung, die mit einer Animation in die Box getippt wird. Die Meldung bleibt editierbar — committen Sie erst, wenn Sie zufrieden sind;
- **Diff-Analyse** — der Funken-Button in der Kopfzeile der Diff-Ansicht analysiert das aktuelle Diff;
- **Konflikt-KI-Agent** — der KI-Modus des Merge-Editors des Rebase löst Hunks einzeln mit Erklärungen und Konfidenz (siehe Konflikte lösen).

![Die KI-generierte Commit-Meldung, in die Commit-Box getippt](images/git-llm-commit-message.jpg)

> Das LLM liest Repository-Inhalte nur, um Text zu erzeugen; Commits und Merges geschehen immer erst, wenn Sie sie bestätigen.
