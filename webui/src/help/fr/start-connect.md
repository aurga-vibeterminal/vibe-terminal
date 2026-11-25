Connectez un client à ce serveur avec le QR code d’appairage de l’interface Web — ou en saisissant les informations à la main. Tout ce dont un client a besoin (adresse, port, clé secrète) est affiché dans la section Server.

## Préparer le serveur

- Ouvrez la section `Server` et assurez-vous qu’une `Secret Key` est définie ; cliquez sur le bouton dé pour en générer une. Régénérer la clé invalide chaque client appairé, qui doit se réappairer.
- Dépliez le bloc `QR Code` et choisissez l’`Network Interface` et l’`IP Version` (IPv4 / IPv6) que les autres machines doivent utiliser pour joindre ce serveur.
- Si un domaine DDNS est configuré, vous pouvez basculer `Connection Type` sur `DDNS Domain` pour que le QR code intègre le domaine au lieu d’une adresse IP.
- L’`Pairing Preview` affiche l’URL de connexion exacte (`wss://host:port`) que le client composera.
- Cliquer sur le `QR code` copie une chaîne de partage chiffrée dans le presse-papiers — les clients l’utilisent pour un remplissage automatique en un geste.

## Client de bureau

- Cliquez sur le bouton du nom du serveur dans la barre de titre et choisissez `Servers` dans le menu pour ouvrir la boîte de dialogue Server Manager.

![Client de bureau — le menu serveur ouvert en cliquant sur le bouton du nom du serveur](images/start-connect-server-menu.jpg)

- Cliquez sur le bouton `+` à côté de la liste des serveurs pour lancer le formulaire Add Server.
- Si la chaîne de partage est dans votre presse-papiers, une icône de presse-papiers apparaît en haut à droite du formulaire — cliquez dessus pour déchiffrer la chaîne et remplir automatiquement `Title`, `Address` et `Secret Key`.
- Sinon, remplissez les champs à la main : un nom d’affichage, l’URL de connexion (`wss://host:port`, adresses IPv6 entre crochets) et la clé secrète.
- Cliquez sur `Save`, puis choisissez le nouveau serveur pour ouvrir une session.

![Client de bureau — Server Manager avec le formulaire Add Server](images/start-connect-add-server.jpg)
