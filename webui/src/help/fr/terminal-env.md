Les variables d’environnement vous permettent de maintenir des ensembles groupés de variables par projet (`ANDROID_HOME`, clés d’API, réglages de proxy) et de les appliquer ou les retirer sur n’importe quelle session de terminal en un clic. Elles vivent dans la configuration du serveur ; les valeurs sont chiffrées au repos.

## Maintenance dans l’interface Web

La section `Environment Variables` les organise en arbre :

- **Groupes** — purs conteneurs, imbriquables (clic droit → `New Group`), pour organiser par projet ou scénario ;
- **Collections** — les feuilles de l’arbre et les véritables porteuses de variables (`New Leaf`) ; seules les collections peuvent être appliquées à un terminal ;
- Les paires clé/valeur de chaque collection s’éditent via `Edit Variables` (`Add Variable`) ; les valeurs sont masquées par défaut ;
- Les collections peuvent être dupliquées pour dériver des variantes (le même ensemble avec une clé changée).

![La page Environment Variables — l’arbre groupes/collections](images/terminal-env-tree.jpg)

## Appliquer à un terminal

Clic droit dans un terminal → `Envs` :

- le sous-menu reflète l’arbre de l’interface Web ; dépliez une collection et choisissez `Apply` pour injecter ses variables dans la session courante, ou `Clear` pour les retirer ;
- les processus récemment démarrés voient les variables (les programmes de premier plan déjà en cours non) ;
- les collections apparaissent repliées sous leurs groupes ; l’élément `Envs` entier se cache quand l’arbre n’a pas de collections.

![Clic droit terminal — appliquer et retirer dans le sous-menu Envs](images/terminal-env-menu.mp4)
