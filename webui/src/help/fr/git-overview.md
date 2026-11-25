Le panneau Git intègre le contrôle de versions dans le client : stager, commettre, gérer les branches, worktrees, rebase interactif, résolution de conflits et poussée — sans quitter votre fenêtre de terminal. Tout opère sur le dépôt Git de la session de terminal courante : quel que soit le répertoire où la session tourne, c’est le dépôt que montre le panneau.

## L’ouvrir

Cliquez sur l’onglet `Git` dans la barre d’activités de gauche (les données du dépôt n’apparaissent que quand la session active est dans un dépôt Git). Le dépôt d’un onglet suit sa session active ; le fil d’Ariane du dépôt permet de sauter entre dépôts parent et enfant, et les sous-modules sont listés à part.

![Le panneau Git — changements, historique et branches](images/git-overview-panel.jpg)

## Ce qu’il sait faire

- **Staging et commits** — stage/unstage par fichier ou en masse, abandonner des changements, commettre avec un message, messages de commit générés par IA (voir Assistance LLM) ;
- **Historique** — historique des commits et par fichier, comparer et restaurer entre révisions ;
- **Branches** — checkout, créer et supprimer des branches locales/distantes et des étiquettes ;
- **Worktrees** — extraire une branche dans un arbre de travail isolé (un nouvel onglet de terminal s’ouvre automatiquement), la fusionner au retour (voir Worktrees) ;
- **Rebase interactif** — réordonner/écraser/éditer visuellement la file de commits avec résolution de conflits (voir Rebase interactif) ;
- **Résolution de conflits** — un éditeur de fusion à quatre modes, dont un agent IA qui résout les hunks un à un (voir Résoudre les conflits) ;
- **Push et divers** — une boîte de push complète, les stashes, la gestion du HEAD détaché (voir Push, checkout et plus).

Les pages suivantes couvrent chaque domaine dans cet ordre.
