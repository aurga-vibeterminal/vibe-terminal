Connecter une machine au relais prend deux champs : l’adresse du relais et sa clé. Voir la page précédente, Présentation, pour ce qu’est la fonction et comment les pièces s’emboîtent.

## Installer le serveur relais

1. Ouvrez la section `Server` sur la machine qui fera office de relais.
2. Assurez-vous qu’une `Secret Key` est définie — le bouton dé en génère une. La même clé apparie clients et serveurs partagés.
3. Dépliez le bloc `QR Code` et choisissez l’`Network Interface` / l’`IP Version` que clients et serveurs partagés doivent employer pour l’atteindre.

## Connecter un serveur partagé

Les coordonnées du relais voyagent via le presse-papiers — rien à retaper :

1. Sur l’interface Web du serveur relais, cliquez le `QR code`. Ceci copie une chaîne de partage chiffrée (adresse, port, clé secrète) dans le presse-papiers.
2. Ouvrez l’interface Web sur la machine à partager, section `Server`, et cliquez dans le champ `Relay Address`.
3. Le navigateur fait apparaître une boîte demandant si ce site peut accéder au presse-papiers — cliquez `Allow` :

![Navigateur — la boîte d’autorisation d’accès au presse-papiers affichée quand le champ a le focus](images/relay-allow-paste.jpg)

4. Avec l’accord donné, le presse-papiers est vérifié : quand il contient la chaîne de partage, un bouton de collage apparaît à côté du champ — cliquez-le pour déchiffrer et remplir automatiquement `Relay Address` et `Relay Secret Key` :

![Interface Web — adresse de relais et clé secrète remplies automatiquement depuis la chaîne collée](images/relay-paste-share.jpg)

5. Vérifiez les valeurs et cliquez `Apply` — l’enregistrement reconnecte le relais en amont.

![Interface Web — les champs de relais remplis avec le bouton Apply](images/relay-apply-share.jpg)

> Lire le presse-papiers exige un contexte sécurisé (HTTPS ou localhost). En HTTP simple le bouton de collage reste caché ; saisissez à la main l’adresse du relais et la clé.

Par défaut, la connexion en amont est chiffrée mais non vérifiée. Pour épingler aussi le certificat du relais, voir Vérification stricte côté relais sous Certificats et TLS.

## Client de bureau

### Onglets des serveurs partagés

Les onglets de terminaux tournant sur un serveur partagé portent deux marqueurs : une bande colorée sur le bord gauche de l’onglet — une couleur par serveur — et l’icône de l’OS distant (Windows / Apple / Linux) à côté du point d’état. Les onglets locaux du serveur relais lui-même n’ont pas de marqueurs.

![Client de bureau — marqueurs d’onglets des terminaux de serveurs partagés](images/relay-tab-indicator.jpg)

### Profils de terminal par serveur

Le menu des profils (barre de titre) groupe les profils par serveur : d’abord ceux du serveur relais connecté, puis un groupe repliable par serveur partagé, nommé d’après la machine. Dépliez un groupe et choisissez un profil pour ouvrir un terminal sur ce serveur. Le menu suit le groupe d’onglets sélectionné dans la barre de titre.

![Client de bureau — menu des profils avec groupes par serveur](images/relay-profile-menu.jpg)

### Groupage et fenêtres séparées

Le sélecteur de groupes de la barre de titre liste les groupes d’onglets de chaque serveur ; le groupe par défaut d’un serveur partagé s’affiche comme `Ungrouped (Nom du serveur)`. Le sous-menu de chaque groupe liste ses onglets pour basculer vite et offre `Move Group to New Window`, qui détache le groupe entier dans sa propre fenêtre — utile pour dédier une fenêtre ou un moniteur à une machine.

![Client de bureau — menu du sélecteur de groupes avec l’action déplacer vers une nouvelle fenêtre](images/relay-group-selector.jpg)
