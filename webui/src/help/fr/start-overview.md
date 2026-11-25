Vibe Terminal est un hub de terminaux géré à distance pour le développement embarqué et serveur. Un **serveur** tourne sur chaque machine sur laquelle vous travaillez — un poste de développement, un serveur de compilation, une carte embarquée — et les **clients** s’y connectent depuis une application de bureau ou la ligne de commande. Ce guide couvre les deux côtés : l’interface Web qui configure un serveur, et les clients qui travaillent avec lui.

## Comment tout s’articule

- Tout tourne sur le **serveur** — les sessions de terminal y vivent : fermer un client, perdre le réseau ou réorganiser les fenêtres n’interrompt jamais une commande en cours ; reconnectez-vous et reprenez là où vous en étiez ;
- Les clients ouvrent une connexion WebSocket chiffrée unique (`wss://host:port` ; `wss://localhost:5153` pour le serveur local) et s’authentifient avec la **clé secrète** du serveur ; l’appairage transmet la clé via un QR code ou une chaîne de partage chiffrée (voir Connecter un client) ;
- L’**interface Web** est la page de configuration du serveur lui-même — profils, addons, fournisseurs LLM, relais, redirection de ports et plus. Ce n’est pas un terminal : le travail quotidien se fait dans le client de bureau ou `vtermc`.

## Composants

- **Serveur** — tourne sur la machine cible ; héberge les sessions de terminal et fournit aux clients connectés les fichiers, Git, le bureau à distance ainsi que les fonctions vocales et IA ;
- **Client de bureau** — l’espace de travail natif : terminaux avec divisions, groupes d’onglets et fenêtres détachées, un explorateur et éditeur de fichiers, des outils Git, la saisie vocale, l’historique des conversations des agents de code, le bureau à distance et le débogage distant ;
- **CLI `vtermc`** — lancez et rattachez des sessions depuis n’importe quel terminal classique, via SSH ou sur une machine headless ; partage sa liste de serveurs avec le client de bureau (voir Client en ligne de commande) ;
- **Interface Web** — réglages du serveur lui-même dans le navigateur, organisés en sections dans la barre latérale gauche (voir Utiliser l’interface Web).

## Icône de la zone de notification

Sur les machines dotées d’une session de bureau, le serveur affiche une icône dans la zone de notification pendant son exécution :

![Menu de l’icône de la zone de notification — Open ramène au premier plan un client en cours d’exécution ou en lance un, Settings ouvre l’interface Web et l’entrée vocale démarre ou arrête le serveur ASR](images/start-tray-menu.jpg)

- **Open** — ramène au premier plan un client de bureau en cours d’exécution, ou en lance un quand aucun ne tourne (un double-clic sur l’icône fait de même) ;
- **Settings** — ouvre l’interface Web du serveur dans le navigateur (voir Utiliser l’interface Web) ;
- **Start/Stop Vibe Voice** — démarre ou arrête le serveur ASR sans ouvrir l’interface Web, à l’image des boutons Start/Stop de la section `Voix` ; l’entrée apparaît quand la reconnaissance vocale est activée, et son libellé suit l’état (p. ex. `Stop Vibe Voice (model on device)` tant que le modèle est chargé) ;
- **Exit** — arrête le serveur.

Tant que des répertoires sont publiés via HTTP, une entrée `HTTP Servers` permet d’arrêter chaque service en un clic (voir Publier avec un serveur HTTP).

## Carte des fonctionnalités

Le reste de ce guide reflète les chapitres de la barre latérale :

- **Terminaux** — création, division et organisation des terminaux ; profils, WSL et série UART ; commandes rapides et variables d’environnement ; piloter les terminaux depuis des agents externes avec MCP ; revoir les conversations des agents de code ; confirmation automatique des invites interactives ;
- **Fichiers** — projets récents, l’explorateur de fichiers côté serveur, le téléversement et le téléchargement, et la publication d’un répertoire via HTTP (voir Explorateur de fichiers, Téléverser et télécharger et Publier avec un serveur HTTP) ;
- **Git** — staging, commits assistés par IA, worktrees, rebase interactif, résolution de conflits et push ;
- **Bureau à distance et simulateurs** — regarder et contrôler les écrans du serveur, les appareils et émulateurs Android, et le simulateur iOS ;
- **Saisie vocale** — reconnaissance vocale côté serveur avec dictée par pression sur une touche et perfectionnement par un LLM ;
- **Relais** — joindre des machines derrière un NAT ou des IP dynamiques via un point d’entrée public unique ;
- **Accès distant** — domaines DDNS et redirection de ports TLS pour les services tournant sur le serveur ;
- **Fonctions IA** — fournisseurs LLM en addons, assignés par fonction, plus l’intégration MCP (voir LLM et MCP).

## Par où commencer

1. Ouvrez la section `Server` de l’interface Web, définissez une `Secret Key` et appairez un client — Connecter un client détaille la configuration par QR code et manuelle ;
2. Parcourez les sections de l’interface Web pour configurer profils, addons et fonctions — voir Utiliser l’interface Web ;
3. Ouvrez un terminal sur le serveur et parcourez les chapitres ci-dessus selon vos besoins.
