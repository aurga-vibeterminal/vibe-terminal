Le rebase interactif transforme `git rebase -i` en un flux visuel : réordonner la file de commits par glisser, appliquer les commits un à un, résoudre les conflits visuellement — pas de ligne de commande nécessaire.

## Démarrer

Ouvrez le volet des branches et tapez dans la boîte `Filter branches...` pour trouver la branche cible, puis clic droit dessus → `Rebase Current Branch onto '{branch}'` pour rebaser la branche courante sur la cible. Le panneau bascule en vue de rebase, qui traverse quatre phases :

![Recherche de branche dans le volet, puis démarrage du rebase sur la cible](images/git-rebase-search.jpg)

## 1. Éditer la file de commits

Les commits de votre branche que la cible n’a pas forment une file :

- **glisser** pour les réordonner ;
- chaque commit a un menu d’action : `pick` (garder) / `squash` (replier dans le précédent, fusionner les messages) / `fixup` (replier, jeter le message) / `edit` (s’arrêter pour amender) / `drop` (jeter) ;
- les commits squashés et droppés se replient visuellement.

![La file de commits du rebase — réordonner par glisser, actions par commit](images/git-rebase-queue.jpg)

## 2. Appliquer

`Start Rebase` rejoue la file sur la cible commit par commit avec une barre de progression en direct ; sans conflit elle va droit au bout. `Abort` fait marche arrière à tout moment.

## 3. Conflits (s’il y en a)

Un conflit pendant l’application bascule vers la vue de conflits — une liste de fichiers plus l’éditeur de fusion et ses quatre modes (`Merge` / `Split` / `Raw` / `AI`), où chaque hunk en conflit reçoit sa propre décision (`Ours` / `Theirs` / `Both`… ; voir Résoudre les conflits). Une fois chaque fichier résolu, `Continue` reprend le rebase.

![La vue de conflits — décisions de fusion par hunk](images/git-rebase-resolve-conflict.jpg)

## 4. Terminé

À l’achèvement, le panneau confirme `Branch '{branch}' was rebased onto '{target}'` et montre la carte de push — le rebase a réécrit l’historique, la branche distante doit être mise à jour en force :

- `Force Push with Lease` — recommandé : n’écrase que lorsque personne d’autre n’a poussé et n’utilise jamais un `--force` simple ;
- `Not now` — sauter le push ; le rebase local est déjà complet.

![La page de fin — Force Push with Lease ou ignorer le push](images/git-rebase-done.jpg)

> La session reste utilisable pendant le rebase ; fermer le panneau n’interrompt pas un rebase en cours.
