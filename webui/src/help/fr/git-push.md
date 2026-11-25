Le menu contextuel de la liste des branches concentre le push, le checkout et les opérations quotidiennes de branches.

## Push

Clic droit sur une branche → `Push '{branch}'...` ouvre la boîte de push :

![Clic droit branche — l’entrée Push](images/git-push-contextmenu.jpg)

- elle montre le mappage de cibles `local → remote` ; chaque nom peut être changé (pousser vers une branche distante existante sous un autre nom demande confirmation) ;
- options : `Push all branches`, `Set upstream` (lier l’amont au premier push), `Push tags`, `Dry run` ;
- le `Force` réécrivant l’historique et le plus sûr `Force with lease` (refuse quand quelqu’un d’autre a poussé — préféré). Les opérations réécrivant l’historique, comme un push après un rebase, exigent l’un de ceux-ci ;
- un indice de risque sur les push en force est affiché dans la boîte.

![La boîte de push — cibles local → remote et cases d’options](images/git-push-dialog.jpg)

## Checkout

Clic droit sur une branche → `Checkout` pour changer de branche — les branches locales se checkotent directement, une branche distante ou une étiquette ouvre la boîte de checkout :

![Clic droit branche — l’entrée Checkout](images/git-checkout-contextmenu.jpg)

- `New branch name` — la branche locale à créer, préremplie depuis la référence ;
- `Create new local branch` (`-B`), `Track remote as upstream` (`--track`, branches distantes seulement), `Force` (`-f`), `Merge` (`--merge`).

![La boîte de checkout — nom de branche et options](images/git-checkout-dialog.jpg)

Le répertoire de travail de la session suit le changement ; le terminal et le panneau Git reflètent la nouvelle branche immédiatement.

## HEAD détaché

Extraitre un commit nu (p. ex. extraire un commit en parcourant l’historique) détache le HEAD. Le commit suivant ouvre alors la boîte `Detached HEAD` : choisissez une branche existante où commettre, `+ Create new branch…` sur place, ou `Skip` pour l’instant.

## Autres opérations

- **Stash** — stash toutes les modifications sur la pile (message facultatif) depuis l’en-tête des changements ; la liste `STASHES` gère `Pop` (appliquer et retirer) et `Drop` (jeter, irréversible) ;
- **Supprimer branches/worktrees** — via le menu contextuel de l’entrée, avec confirmations d’action irréversible ;
- **Historique et restauration** — `HISTORY` liste les commits ; clic droit pour l’historique d’un fichier, un diff contre l’arbre de travail ou une restauration vers une révision ;
- **Copier** — noms de branches et empreintes de commit se copient en un clic, pratique pour coller à un agent ou un collègue.
