Les worktrees extraient plusieurs branches d’un même dépôt dans des répertoires séparés : développez en parallèle sans toucher l’arbre de travail principal, puis fusionnez la branche en retour et nettoyez. Vibe Terminal transforme tout le cycle en pointer-cliquer.

## Créer un worktree

1. Dans la liste des branches du panneau Git, clic droit sur la branche cible (locale ou distante) → `New Worktree` ;
2. la boîte nomme le nouveau worktree d’après la branche — modifiez-le à volonté ; confirmez, et le serveur crée le répertoire du worktree et y extrait la branche ;
3. **un nouvel onglet de terminal s’ouvre automatiquement** avec son répertoire de travail réglé sur le nouveau worktree — commencez immédiatement à développer en isolation pendant que l’arbre principal reste sur sa branche.

![La boîte New Worktree — le nom vaut la branche par défaut](images/git-worktree-create.jpg)

![Le nouvel onglet ouvert pour le worktree fraîchement créé](images/git-worktree-terminal.jpg)

## Travailler en parallèle

Chaque worktree est un répertoire de travail totalement indépendant : builds et tests de la branche principale et de la branche de fonctionnalité tournent côte à côte dans des onglets différents sans interférence. Les branches extraites dans un worktree portent un marqueur de worktree montrant leur chemin dans la liste des branches, et committer et pousser fonctionnent depuis l’onglet du worktree comme sur n’importe quelle branche.

Le clic droit sur une branche de worktree ajoute deux entrées propres aux worktrees : `Reveal in File Explorer` (sessions locales) ouvre son répertoire dans le gestionnaire de fichiers du système, et `Open in Terminal` y ouvre un autre onglet.

## Fusionner en retour et nettoyer

1. Commitez et poussez dans l’onglet du worktree comme d’habitude ;
2. basculez l’onglet principal sur la branche cible (p. ex. `main`), clic droit sur la branche de fonctionnalité → `Merge into Current Branch` pour rapatrier le travail ;

   ![Fusionner la branche du worktree en retour dans la branche principale](images/git-worktree-merge.jpg)

3. une fois fusionné, clic droit sur la branche → `Delete Worktree` : ceci supprime ensemble le répertoire du worktree et sa branche. Assurez-vous d’abord que tout est fusionné ou poussé — la boîte avertit encore et l’action est irréversible.

   ![La confirmation Delete Worktree — répertoire et branche supprimés ensemble](images/git-worktree-delete.jpg)

