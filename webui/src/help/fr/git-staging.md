La zone `CHANGES` du panneau Git couvre la boucle de commit quotidienne : relire les changements, stager, abandonner, stash et commettre avec un message.

## Stager et unstager

- les changements non stagés sont listés sous `CHANGES`, chaque fichier marqué d’un badge d’état — `M` modifié, `A` ajouté, `D` supprimé, `?` non suivi, `!` conflit (les lignes de sous-module portent un tag `SUB` supplémentaire) ;
- survoler une ligne révèle des boutons d’action en ligne : `+` stade un fichier non stagé, `−` unstade un fichier stagé, et `×` sur une ligne non stagée l’abandonne ; les boutons `Stage All Changes` / `Unstage All Changes` des en-têtes de section traitent tout d’un coup ;
- `Ctrl`-clic bascule des fichiers en sélection multiple et `Shift`-clic sélectionne une plage — un bouton d’action s’applique alors à chaque fichier sélectionné ;
- cliquer un nom de fichier ouvre son diff pour relire le changement avant de décider ;
- le clic droit sur un fichier offre `View File History`, `Stash This File`, `Copy Full Path` / `Copy Relative Path`, `Reveal in File Explorer` et un raccourci `cd` qui fait sauter le terminal au dossier du fichier.

![La section STAGED CHANGES — fichiers stagés avec boutons Unstage par ligne](images/git-staging-staged.jpg)

## Committer

1. Stager ce qui doit entrer ;
2. taper un message dans la boîte de commit (`Ctrl+Entrée` commet) ou cliquer le bouton IA pour en générer un (voir Assistance LLM) ;
3. cliquer `Commit`.

![La boîte de commit — saisie du message avec le bouton de génération IA](images/git-staging-commit.jpg)

Après le commit, survolez l’en-tête d’historique — la barre avec le nom de branche au-dessus de l’historique des commits — et les boutons `Fetch`, `Pull` et `Push` apparaissent en fondu à droite. `Push` envoie aussitôt la branche courante vers son amont ; pour des options comme `Force with lease`, utilisez plutôt la boîte de push de la liste des branches (voir Push, checkout et plus).

![Survoler l’en-tête d’historique révèle les boutons Fetch, Pull et Push](images/git-staging-push.jpg)

## Stasher

- le bouton archive dans l’en-tête `CHANGES` ouvre la boîte `Stash All Changes`, où un message facultatif décrit le stash ;
- le clic droit sur des fichiers non stagés sélectionnés offre `Stash This File` (ou `Stash <n> Selected Files` pour une sélection multiple) pour stasher juste ces chemins — les lignes de sous-module sont exclues ;
- les stashes se collectent sous `STASHES` comme entrées `WIP on <branche>` ; déplier l’une liste les fichiers capturés, et cliquer un fichier montre son diff ;
- survoler une ligne de stash montre `Apply` (restaurer et garder), `Pop` (appliquer et retirer) et `Drop` (retirer sans appliquer) — `Pop` et `Drop` demandent confirmation d’abord, et le menu contextuel ajoute `View Diff` ;
- la section `STASHES` reste cachée tant que la liste de stashes est vide.

![La boîte Stash All Changes avec son message facultatif](images/git-staging-stash-all.jpg)

![La section STASHES — un stash déplié vers ses fichiers, avec les boutons Apply, Pop et Drop](images/git-staging-stashes.jpg)

![Pop applique le stash et le retire de la liste après une confirmation](images/git-staging-stash-pop.jpg)

## Abandonner

`Discard Changes` sur un fichier non stagé jette tous ses changements non commités — il y a une boîte de confirmation et l’action **ne peut pas être annulée**.

> Un commit n’inclut que le contenu stagé ; les changements non stagés restent dans l’arbre de travail.
