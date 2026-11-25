Worktrees checken mehrere Branches eines Repositorys in getrennte Verzeichnisse aus: parallel entwickeln, ohne den Haupt-Arbeitsbaum anzufassen, dann den Branch zurückmergen und aufräumen. Vibe Terminal verwandelt den ganzen Zyklus in Klickarbeit.

## Einen Worktree erstellen

1. Klicken Sie in der Branchliste des Git-Panels mit der rechten Maustaste auf den Ziel-Branch (lokal oder remote) → `New Worktree`;
2. Der Dialog benennt den neuen Worktree nach dem Branch — bearbeiten Sie ihn nach Belieben; bestätigen Sie, und der Server erstellt das Worktree-Verzeichnis und checkt den Branch dort aus;
3. **Ein neuer Terminal-Tab öffnet sich automatisch** mit dem Arbeitsverzeichnis des neuen Worktrees — beginnen Sie sofort isoliert zu entwickeln, während der Haupt-Arbeitsbaum auf seinem Branch bleibt.

![Der Dialog New Worktree — der Name ist standardmäßig der Branch](images/git-worktree-create.jpg)

![Der für den frisch erstellten Worktree geöffnete neue Tab](images/git-worktree-terminal.jpg)

## Parallel arbeiten

Jeder Worktree ist ein völlig unabhängiges Arbeitsverzeichnis: Builds und Tests für den Haupt- und den Feature-Branch laufen in verschiedenen Tabs nebeneinander ohne Störung. In einem Worktree ausgecheckte Branches tragen in der Branchliste eine Worktree-Markierung mit ihrem Pfad, und Committen und Pushen funktionieren direkt aus dem Tab des Worktrees wie in jedem Branch.

Der Rechtsklick auf einen Worktree-Branch ergänzt zwei Worktree-eigene Einträge: `Reveal in File Explorer` (lokale Sitzungen) öffnet sein Verzeichnis im System-Dateimanager, und `Open in Terminal` öffnet dort einen weiteren Tab.

## Zurückmergen und aufräumen

1. Committen und pushen Sie im Tab des Worktrees wie üblich;
2. Schalten Sie den Haupt-Tab auf den Ziel-Branch (z. B. `main`) und klicken Sie mit der rechten Maustaste auf den Feature-Branch → `Merge into Current Branch`, um die Arbeit herüberzuholen;

   ![Den Worktree-Branch zurück in den Haupt-Branch mergen](images/git-worktree-merge.jpg)

3. Nach dem Merge: Rechtsklick auf den Branch → `Delete Worktree`: das entfernt das Worktree-Verzeichnis und seinen Branch zusammen. Stellen Sie vorher sicher, dass alles gemergt oder gepusht ist — der Dialog warnt erneut und die Aktion ist unumkehrbar.

   ![Die Delete-Worktree-Bestätigung — Verzeichnis und Branch werden zusammen entfernt](images/git-worktree-delete.jpg)

