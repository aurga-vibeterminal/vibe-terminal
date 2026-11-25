Das Git-Panel baut Versionsverwaltung in den Client ein: Stagen, Committen, Branches verwalten, Worktrees, interaktiver Rebase, Konfliktlösung und Pushen — ohne Ihr Terminal-Fenster zu verlassen. Alles arbeitet auf dem Git-Repository der aktuellen Terminal-Sitzung: In welchem Verzeichnis die Sitzung auch läuft, dieses Repository zeigt das Panel.

## Öffnen

Klicken Sie in der linken Aktivitätsleiste auf den Tab `Git` (Repository-Daten erscheinen nur, wenn die aktive Sitzung in einem Git-Repository liegt). Das Repository eines Tabs folgt seiner aktiven Sitzung; Repository-Breadcrumbs lassen Sie zwischen Eltern- und Kind-Repositories hüpfen, Submodule werden separat gelistet.

![Das Git-Panel — Änderungen, Verlauf und Branches](images/git-overview-panel.jpg)

## Was es kann

- **Stagen & Committen** — pro Datei oder gesammelt stagen/entstagen, Änderungen verwerfen, mit Meldung committen, KI-generierte Commit-Meldungen (siehe LLM-Unterstützung);
- **Verlauf** — Commit-Verlauf und Verlauf pro Datei, zwischen Revisionen vergleichen und wiederherstellen;
- **Branches** — lokale/Remote-Branches und Tags auschecken, erstellen und löschen;
- **Worktrees** — einen Branch in einen isolierten Arbeitsbaum auschecken (ein neuer Terminal-Tab öffnet sich automatisch), bei Fertigstellung zurückmergen (siehe Worktrees);
- **Interaktiver Rebase** — die Commit-Warteschlange visuell umordnen/squashen/bearbeiten mit Konfliktlösung (siehe Interaktiver Rebase);
- **Konfliktlösung** — ein Merge-Editor mit vier Modi, inklusive eines KI-Agents, der Hunks einzeln löst (siehe Konflikte lösen);
- **Push & Sonstiges** — ein voll ausgestatteter Push-Dialog, Stashes, Umgang mit losgelöstem HEAD (siehe Push, Checkout & Mehr).

Die folgenden Seiten behandeln jeden Bereich in dieser Reihenfolge.
