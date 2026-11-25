Het rechtsklikmenu van de branchlijst bundelt pushen, uitchecken en de dagelijkse branchbewerkingen.

## Push

Rechtsklik een branch → `Push '{branch}'...` opent het pushvenster:

![Branch-rechtsklik — de Push-invoer](images/git-push-contextmenu.jpg)

- Het toont de doelkoppeling `local → remote`; beide namen zijn wijzigbaar (pushen naar een bestaande externe branch onder een andere naam vraagt om bevestiging);
- Opties: `Push all branches`, `Set upstream` (upstream bij de eerste push koppelen), `Push tags`, `Dry run`;
- De geschiedenis herschrijvende `Force` en de veiligere `Force with lease` (weigert als iemand anders heeft gepusht — de voorkeur). Geschiedenherschrijvende bewerkingen, zoals pushen na een rebase, vereisen één hiervan;
- Een risicohint over force-pushes staat in het venster.

![Het pushvenster — local → remote-doelen en de optievakjes](images/git-push-dialog.jpg)

## Uitchecken

Rechtsklik een branch → `Checkout` om van branch te wisselen — lokale branches checken direct uit, terwijl een externe branch of tag het checkout-venster opent:

![Branch-rechtsklik — de Checkout-invoer](images/git-checkout-contextmenu.jpg)

- `New branch name` — de aan te maken lokale branch, vooraf gevuld vanuit de ref;
- `Create new local branch` (`-B`), `Track remote as upstream` (`--track`, alleen externe branches), `Force` (`-f`), `Merge` (`--merge`).

![Het checkout-venster — branchnaam en opties](images/git-checkout-dialog.jpg)

De werkmap van de sessie volgt de wissel; de terminal en het Git-paneel weerspiegelen de nieuwe branch direct.

## Detached HEAD

Een kale commit uitchecken (bijv. een commit bekijken in de geschiedenis en die uitchecken) ontkoppelt de HEAD. De volgende commit opent dan het venster `Detached HEAD`: kies een bestaande branch om in te committen, `+ Create new branch…` ter plekke, of voorlopig `Skip`.

## Meer bewerkingen

- **Stash** — stash alle wijzigingen op de stapel (bericht optioneel) vanuit de wijzigingskop; de lijst `STASHES` ondersteunt `Pop` (toepassen en verwijderen) en `Drop` (verwerpen, onomkeerbaar);
- **Branches/worktrees verwijderen** — via het rechtsklikmenu op de invoer, beide met bevestigingen voor onomkeerbare acties;
- **Geschiedenis & herstellen** — `HISTORY` somt commits op; rechtsklik voor bestandsgeschiedenis, diff tegen de werkboom, of herstellen naar een revisie;
- **Kopiëren** — branchnamen en commit-hashes kopiëren met één klik, handig om naar een agent of collega te plakken.
