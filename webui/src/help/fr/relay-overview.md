Le relais résout un problème : la plupart des machines sont derrière un NAT ou des IP dynamiques, donc les clients ne peuvent pas les appeler chacune directement. Laissez une machine publiquement joignable servir de point d’entrée du relais ; chaque autre machine appelle vers elle. Un client maintient alors une connexion unique vers le relais et atteint à travers lui les terminaux de chaque serveur partagé.

## Architecture

![Architecture du relais — chaque client garde une connexion vers le relais ; les serveurs partagés appellent le relais en amont](diagram/relay)

- **Serveur relais** — le point d’entrée public auquel clients et serveurs partagés se connectent. Laissez son `Relay Address` vide.
- **Serveur partagé** — une machine dont le `Relay Address` pointe vers le relais. Elle appelle en amont et partage ses terminaux à travers lui. La bande colorée sur chaque nœud partagé du schéma reflète la bande que le client de bureau peint sur les onglets de ce serveur — une couleur par serveur.
- Les connexions partent toujours de l’intérieur : les serveurs partagés appellent le relais, donc aucun transfert de port ni IP publique n’est nécessaire chez eux.

## Ce que voit le client

Une fois connecté au relais, chaque serveur partagé se comporte comme une machine locale :

- les onglets de terminal portent une bande colorée et l’icône de l’OS distant, identifiant la machine d’un coup d’œil (voir le Guide d’installation) ;
- le menu des profils groupe les terminaux par serveur — ouvrez un terminal sur n’importe quelle machine directement ;
- les groupes d’onglets peuvent être détachés dans leur propre fenêtre, dédiant une fenêtre à une machine.

## Modèle de sécurité

Clients et serveurs partagés s’appairent avec la même clé secrète ; les connexions sont tunnellisées à travers le relais. La clé voyage dans le QR code d’appairage / la chaîne de partage, jamais en clair.

Les instructions pas à pas sont sur la page suivante, Guide d’installation.
