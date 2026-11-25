Les terminaux sont le cœur de Vibe Terminal : chaque onglet héberge une ou plusieurs sessions de terminal, peut être divisé en volets ou déplacé dans sa propre fenêtre. Les terminaux atteignent le serveur local ainsi que les serveurs distants et partagés via un relais.

## Créer des terminaux

- Cliquez sur le bouton `+` de la barre de titre pour ouvrir un onglet avec le profil par défaut ;
- Cliquez sur le chevron à côté de `+` pour le menu des profils : profils de terminal groupés par serveur (local, connectés, partagés) — dépliez un groupe et choisissez un profil pour ouvrir un terminal sur ce serveur ;
- Faites glisser un dossier local sur la barre de titre et déposez-le pour ouvrir un nouvel onglet avec ce répertoire de travail (serveur local uniquement ; l’élément déposé doit être un dossier).

![Barre de titre — le bouton nouvel onglet et le menu des profils](images/terminal-usage-creation.jpg)

## Capture de la souris et menu contextuel

- Un clic droit dans un terminal ouvre son menu contextuel : coller, diviser en volets, commandes rapides, ensembles de variables d’environnement, copie de tout le texte, lignes de configuration MCP et plus, selon la session ;
- Les applications qui capturent la souris (opencode, pi, vim, tmux, htop, …) reçoivent les événements de pointeur à la place — maintenez `Shift` pour conserver les opérations habituelles pendant qu'une telle application tourne :
  - **Copier du texte** : maintenez `Shift` et sélectionnez le texte, puis cliquez ailleurs — `Shift` toujours maintenu — pour désélectionner ; la sélection est copiée dans le presse-papiers ;
  - **Menu contextuel** : `Shift` + clic droit ou `Ctrl` + clic droit ouvre le menu contextuel ; un clic droit pendant que le bouton gauche est maintenu l'ouvre également (toute sélection est d'abord effacée) ;
  - **Liens** : `Shift` + clic suit les liens normalement.

## Divisions et organisation par glisser-déposer

- **Division** : clic droit dans un terminal → `Split` → `Left` / `Right` / `Top` / `Bottom` divise l’onglet en volets, chacun un terminal indépendant ;

  ![Diviser un onglet — le menu Split du clic droit et les volets obtenus](images/terminal-usage-split.gif)

- **Fusion par glisser** : faites glisser un onglet sur un autre terminal et des zones d’ancrage apparaissent (gauche/droite/haut/bas) ; déposez pour fusionner l’onglet dans l’onglet cible comme nouveau volet ;

  ![Fusion par glisser — un onglet déposé sur un autre terminal devient un nouveau volet](images/terminal-usage-merge.mp4)

- **Détacher vers une fenêtre** : faites glisser un onglet hors de la bande d’onglets (ou sur une autre fenêtre) et tout l’onglet quitte sa fenêtre pour devenir une nouvelle. Les sessions tournent toujours sur le serveur — divisions, fusions et glisser-déposer n’interrompent jamais une commande en cours.

## Onglets et groupes d’onglets

- **Onglet** — un onglet de terminal, éventuellement avec plusieurs volets divisés ;
- **Groupe d’onglets** — un conteneur organisant les onglets. Par défaut les groupes suivent les serveurs : les onglets du serveur local vivent dans `Ungrouped`, et chaque serveur partagé reçoit un groupe `Name (Server)` ; les groupes personnalisés sont pris en charge ;
- **Groupes et fenêtres** — chaque fenêtre affiche un groupe actif via le sélecteur de groupes de la barre de titre (ou `All`) ; `Move Group to New Window` détache un groupe entier dans sa propre fenêtre, dédiant une fenêtre ou un moniteur à une machine ;
- **Groupes et serveurs** — un groupe mémorise son serveur, donc changer de groupe refocalise l’espace de travail d’un serveur ; le menu des profils suit le groupe actif et ne montre que les profils des serveurs pertinents.

![Le sélecteur de groupes — liste des groupes et déplacement vers une nouvelle fenêtre](images/terminal-usage-groups.jpg)
