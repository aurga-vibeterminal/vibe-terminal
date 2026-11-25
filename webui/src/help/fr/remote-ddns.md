Le haut débit résidentiel et de bureau vient typiquement avec une IP publique dynamique qui change sans prévenir. Le DDNS garde un domaine fixe pointé vers l’IP actuelle de la machine — clients et autres serveurs n’ont que le nom à retenir, jamais l’adresse.

## Comment ça marche

![Architecture DDNS — le gestionnaire détecte périodiquement l’IP locale, l’addon met à jour l’enregistrement DNS via l’API du fournisseur, les clients se connectent par domaine](diagram/ddns)

Tout le pipeline tourne automatiquement en arrière-plan du serveur :

1. Le **gestionnaire DDNS** se rafraîchit à cycle fixe (toutes les 5 minutes par défaut, réglable via la variable d’environnement `TERMINAL_SERVER_DDNS_POLL_INTERVAL_SECS`) ; l’enregistrement de la configuration déclenche un rafraîchissement immédiat, et rien ne tourne tant que le service est désactivé ;
2. La **détection d’IP locale** lit d’abord l’interface réseau choisie, retombe sur une découverte par routage, et filtre le résultat selon le réglage `IP Family` (IPv4 / IPv6 / dual) ;
3. L’**addon DDNS** — un script JS en bac à sable — reçoit la configuration déchiffrée et l’IP courante et appelle l’API du fournisseur DNS en HTTP. L’addon Cloudflare intégré, par exemple, cherche la zone par domaine et réécrit les enregistrements A / AAAA vers l’IP courante, supprimant les enregistrements périmés quand une famille d’adresses disparaît ;
4. Les **clients** résolvent le domaine vers la dernière IP et se connectent par nom.

## Configuration

Ouvrez la section `DDNS` de l’interface Web :

1. Activez `Enable`. Vous pouvez aussi laisser le service éteint pendant que vous remplissez tout — un service désactivé enregistre sa configuration mais ne tourne jamais ;
2. choisissez un fournisseur sous `Provider` ; `Cloudflare DDNS` est intégré ;
3. remplissez les champs du fournisseur (exemple Cloudflare) :
   - `Email` — l’e-mail du compte Cloudflare ;
   - `API Token` — un jeton avec la permission Zone → DNS → Edit ;
   - `Domain` — le domaine racine de la zone, p. ex. `example.com` ;
   - `Subdomain` — le nom d’enregistrement dans la zone, p. ex. `sub` pour `sub.example.com` ; laisser vide pour la racine de zone ;
4. dans `Network Interface & IP`, choisissez l’`Interface` et l’`IP Family`. `Dual Stack` exige que l’interface ait à la fois une adresse IPv4 et IPv6 ;
5. cliquez `Save`, puis `Sync Now` pour vérifier. `Sync Status` montre la dernière IP poussée et l’heure de synchronisation, avec le message d’erreur de l’addon en cas d’échec.

## Sécurité

Les identifiants du fournisseur (jetons d’API) sont chiffrés avant stockage (`enc-v2:`) et déchiffrés seulement en mémoire pendant l’exécution de l’addon ; les addons s’exécutent dans un runtime JS en bac à sable dont le HTTP sortant est la seule capacité.

## Étendre les fournisseurs

Les fournisseurs sont des addons : un manifeste JSON (déclarant les champs de configuration que l’interface Web transforme en formulaire) plus un script JS exposant `run()`. Le `cloudflare-ddns` intégré est l’implémentation de référence — des fournisseurs personnalisés peuvent être téléversés en zip dans la section `Addons`.

## Combiné aux autres fonctions

- une fois le domaine résolu, le QR code de la section `Server` peut basculer `Connection Type` sur `DDNS Domain`, pour que les clients scannent une connexion par domaine ;
- d’autres serveurs peuvent mettre `wss://your-domain:port` dans leur `Relay Address` pour faire tourner la chaîne de relais via le domaine (voir le chapitre Relais).
