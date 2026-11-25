Il y a deux façons d’utiliser WSL sous Windows : la voie légère ajoute un shell WSL comme profil de terminal, la voie complète installe vterm-server dans WSL. La première ne vous donne qu’un shell ; la seconde déverrouille la gestion de fichiers, le panneau Git, MCP et tout le reste.

## La voie légère : un profil de terminal WSL

Sous Windows, **les profils de terminal WSL se chargent automatiquement** : au démarrage, le serveur détecte `wsl.exe` et énumère les distributions installées, ajoutant une entrée de profil par distribution à la liste (p. ex. `Ubuntu (WSL)` ; une entrée générique `WSL` apparaît quand aucune distribution n’est listée).

Pour en utiliser une, choisissez simplement l’entrée WSL dans le menu des profils de la barre de titre du client — la distribution démarre et vous laisse directement dans son terminal, comme en ouvrir un local.

Limite : ces sessions tournent sur le serveur côté Windows — **l’explorateur de fichiers, le panneau Git, les variables d’environnement et compagnie du client ne fonctionnent pas** (ils opèrent avec la sémantique du système de fichiers Windows et ne voient pas le système de fichiers Linux dans WSL).

![Le menu des profils de terminal — les distributions WSL listées automatiquement](images/terminal-wsl-profile.jpg)

## La voie complète : installer vterm-server dans WSL

Traitez WSL comme sa propre machine Linux et installez le vterm-server headless. Le site web propose un script en une ligne — collez-le dans un terminal WSL :

```bash
curl -fsSL https://vterminal.dev/install.sh | bash
```

Le script automatiquement :

1. Détecte l’environnement WSL/headless et télécharge le vterm-server **headless** dans `~/.local/vterm` ;
2. Installe un **service utilisateur systemd** et le démarre immédiatement — il tourne à chaque connexion (démarrage WSL inclus), aucun démarrage manuel nécessaire ;
3. Ajoute `~/.local/vterm` à votre PATH.

> Quand systemd est désactivé dans WSL, le script vous indique d’ajouter `[boot]` `systemd=true` dans `/etc/wsl.conf`, d’exécuter `wsl --shutdown` côté Windows et de relancer l’installateur.

## Ouvrir la page de configuration

Dans WSL :

```bash
vterm-server --open
```

La commande affiche l’adresse de l’interface Web (`http://localhost:<config-port>/`) et tente de lancer un navigateur (dans WSL via `wslview`, le navigateur par défaut de Windows). Grâce au transfert de localhost de WSL2, **ouvrir la même adresse dans un navigateur Windows** fonctionne tout aussi bien. Tant que le serveur tourne, l’URL affichée embarque un jeton de connexion à usage unique (`?ticket=...`, délivré via une socket locale) : la page s’ouvre déjà authentifiée.

## (Facultatif) Partager le serveur WSL dans le relais hôte

Si la machine Windows fait aussi tourner vterm-server (en tant que relais), le serveur WSL peut s’y attacher pour que le client voie les deux « machines » via une seule connexion locale :

1. Sur l’interface Web du serveur Windows, cliquez sur le QR code pour copier la chaîne de partage ;
2. Ouvrez l’interface Web du serveur WSL (adresse de la section précédente), cliquez sur le bouton de collage qui apparaît à côté du champ `Relay Address` pour remplir automatiquement adresse et clé, puis `Apply` ;
3. Tant que le vterm-server Windows tourne, le client voit les terminaux, fichiers et Git du serveur WSL sous le relais local.

> Avec le réseau NAT par défaut de WSL2, WSL atteint l’hôte Windows via l’adresse de l’hôte vue côté WSL (la passerelle par défaut — le nameserver dans `cat /etc/resolv.conf`) ; en réseau en miroir, `localhost` fonctionne. Voir le chapitre Relais pour les détails du relais.
