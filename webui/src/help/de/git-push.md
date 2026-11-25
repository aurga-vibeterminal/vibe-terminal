Das Rechtsklick-Menü der Branchliste bündelt Pushen, Auschecken und die alltäglichen Branch-Operationen.

## Push

Rechtsklick auf einen Branch → `Push '{branch}'...` öffnet den Push-Dialog:

![Branch-Rechtsklick — der Push-Eintrag](images/git-push-contextmenu.jpg)

- Er zeigt die Zielzuordnung `local → remote`; jeder der beiden Namen lässt sich ändern (ein Push auf einen bestehenden Remote-Branch unter anderem Namen fragt nach Bestätigung);
- Optionen: `Push all branches`, `Set upstream` (Upstream beim ersten Push binden), `Push tags`, `Dry run`;
- Der historieumschreibende `Force` und der sicherere `Force with lease` (verweigert, wenn jemand anderes gepusht hat — bevorzugt). Historieumschreibende Operationen wie ein Push nach einem Rebase erfordern eines davon;
- Ein Risikohinweis zu Force-Pushes steht im Dialog.

![Der Push-Dialog — local → remote-Ziele und die Options-Kontrollkästchen](images/git-push-dialog.jpg)

## Checkout

Rechtsklick auf einen Branch → `Checkout` zum Branch-Wechsel — lokale Branches checken direkt aus, ein Remote-Branch oder Tag öffnet den Checkout-Dialog:

![Branch-Rechtsklick — der Checkout-Eintrag](images/git-checkout-contextmenu.jpg)

- `New branch name` — der zu erstellende lokale Branch, aus dem Ref vorbelegt;
- `Create new local branch` (`-B`), `Track remote as upstream` (`--track`, nur Remote-Branches), `Force` (`-f`), `Merge` (`--merge`).

![Der Checkout-Dialog — Branchname und Optionen](images/git-checkout-dialog.jpg)

Das Arbeitsverzeichnis der Sitzung folgt dem Wechsel; Terminal und Git-Panel spiegeln den neuen Branch sofort.

## Losgelöster HEAD (Detached HEAD)

Das Auschecken eines nackten Commits (z. B. beim Durchstöbern des Verlaufs) löst den HEAD ab. Der nächste Commit öffnet dann den Dialog `Detached HEAD`: wählen Sie einen bestehenden Branch zum Committen, `+ Create new branch…` an Ort und Stelle, oder vorerst `Skip`.

## Weitere Operationen

- **Stash** — alle Änderungen vom Änderungskopf auf den Stapel stashen (Meldung optional); die Liste `STASHES` unterstützt `Pop` (anwenden und entfernen) und `Drop` (verwerfen, unumkehrbar);
- **Branches/Worktrees löschen** — über das Rechtsklick-Menü des Eintrags, beide mit Bestätigungen für unumkehrbare Aktionen;
- **Verlauf & Wiederherstellen** — `HISTORY` listet Commits; Rechtsklick für Dateiverlauf, Diff gegen den Arbeitsbaum oder Wiederherstellen auf eine Revision;
- **Kopieren** — Branchnamen und Commit-Hashes kopieren sich mit einem Klick, praktisch zum Einfügen an einen Agenten oder Kollegen.
