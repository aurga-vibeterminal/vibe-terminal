Quand un rebase ou une fusion rencontre des conflits, le panneau bascule vers la vue de conflits : la liste des fichiers en conflit à gauche, l’éditeur de fusion à droite. Les fichiers en conflit apparaissent aussi dans la liste `CHANGES` avec un badge `!` — en cliquer un saute directement dans cette vue au lieu d’un simple diff. L’éditeur offre quatre modes de résolution et vous pouvez basculer entre eux à tout moment.

![La vue de conflits — liste de fichiers à gauche, éditeur de fusion à droite](images/git-conflicts-view.jpg)

## La liste des fichiers en conflit

Les fichiers sont groupés par état : ceux fusionnés automatiquement avec succès (stagés automatiquement) et ceux exigeant encore une attention manuelle. Cliquez un fichier pour l’ouvrir dans l’éditeur ; la liste se met à jour en direct à mesure que vous résolvez, jusqu’à être vide.

## Les quatre modes de résolution

- **Merge** (défaut) — les conflits sont présentés hunk par hunk ; par hunk choisissez `take ours` / `take theirs` ou éditez le résultat fusionné en ligne ;
- **Split** — un diff est cassé en segments plus fins au niveau ligne, pour adopter les deux côtés morceau par morceau dans un même hunk ;
- **Raw** — édition simple avec marqueurs de conflit `diff3` (`<<<<<<<` / `=======` / `>>>>>>>`) pour l’ajustement à la main ;
- **AI** — un agent résout le fichier hunk par hunk : le LLM analyse chaque hunk en conflit et produit des cartes de solution avec explication et confiance. Vous pouvez taper des instructions pour guider la stratégie (p. ex. « préférer theirs, garder les logs »), mettre en pause, affiner dans un autre mode et reprendre — les hunks déjà résolus sont sautés.

Dans chaque mode la résolution est écrite dans le fichier de votre session et stagée, exactement comme une édition manuelle.

## Poursuivre le rebase

Une fois chaque conflit résolu, le rebase passe au commit suivant ; vous pouvez aussi abandonner à tout moment pour revenir à l’état d’avant rebase.
