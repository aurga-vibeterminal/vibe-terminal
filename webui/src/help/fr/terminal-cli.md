`vtermc` est le client en ligne de commande : il se connecte à un serveur, crée ou rattache des sessions de terminal et gère la liste des serveurs enregistrés — le tout dans un simple terminal, via SSH ou sur une machine headless. Il partage `~/.vibeterminal/client.json` avec le client de bureau, donc les deux voient les mêmes serveurs.

## Démarrer un terminal

Lancez `vtermc` sans indicateur de mode pour créer un nouveau terminal sur le serveur local (`wss://localhost:5153` par défaut) avec le shell détecté, et le diffuser dans le terminal courant. Tapez `exit` (ou `Ctrl+D`) dans le shell distant pour terminer la session.

Les sessions vivent sur le serveur : si la connexion tombe — perte réseau, fermeture de la fenêtre, `vtermc` tué — la session continue de tourner et peut être rattachée plus tard avec `vtermc -s`.

- `vtermc -p <id>` — démarrer avec un profil de terminal précis (la liste des profils est dans la section `Profiles` de l’interface Web) ;
- `vtermc -p` — sans identifiant de profil : le sélecteur liste les profils du serveur de façon interactive ;
- `vtermc -w <dir>` — définir le répertoire de travail de la nouvelle session.

Quand stdin n’est pas un terminal (un tube ou un fichier), `vtermc` passe en mode lot : stdin est envoyé à la session, et la sortie est diffusée jusqu’à la fermeture de la session — pratique depuis des scripts ou des agents :

```sh
echo "make test" | vtermc
```

## Se connecter à un serveur

- `--server wss://host:port` — se connecter à un serveur précis plutôt que le local (adresses IPv6 entre crochets : `wss://[fd00::1]:5153`) ; la variable d’environnement `VTERM_SERVER` définit le même défaut ;
- La poignée de main est répondue avec la clé secrète du serveur : les serveurs ajoutés via `-a`/`-s` portent leur clé dans `client.json` ; le serveur local retombe sur la variable d’environnement `TERMINAL_GUI_SECRET_KEY` (ou une clé en clair dans `server.json`).

## Ajouter des serveurs

`-a` enregistre un serveur dans `client.json` puis quitte — il n’ouvre jamais de terminal. Deux façons :

- `vtermc -a` — invites interactives : `Server name`, `Server address` et `Secret key`. L’adresse doit être `wss://host:port` (une saisie invalide est redemandée ; une adresse vide annule). Un nom vide retombe sur l’adresse ;
- `vtermc -a "vtshare1:..."` — collez la chaîne de partage chiffrée. Cliquer sur le `QR Code` d’appairage dans la section `Server` de l’interface Web la copie dans le presse-papiers ; `vtermc` la déchiffre et remplit automatiquement nom, adresse et clé secrète.

Détails utiles à connaître :

- réajouter une adresse existante met à jour l’entrée en place (son identifiant est conservé) au lieu de la dupliquer ;
- les adresses locales sont refusées — `Local` est toujours implicitement disponible et n’a jamais besoin d’entrée ;
- les entrées sont aussi affichées par le Server Manager du client de bureau.

## Choisir serveurs et sessions

`vtermc -s` ouvre deux sélecteurs plein écran (nécessite un terminal interactif) :

1. **Liste des serveurs** — `Local` en premier, puis chaque serveur géré comme `Name  (adresse)`. Naviguez avec les flèches ou `j`/`k`, sautez avec un chiffre, `Enter` sélectionne, `q` annule ;
2. **Liste des terminaux** — `New Terminal` en premier, suivi des sessions actives de ce serveur avec répertoire de travail, titre et serveur propriétaire. Rattachez une session existante, ou créez-en une nouvelle (qui honore alors `-p` / `-w` comme une simple exécution de `vtermc`).

Le rattachement n’interrompt jamais la session — elle continue de tourner sur le serveur entre les rattachements.

### Supprimer un serveur

Dans la liste des serveurs, placez-vous sur un serveur géré et appuyez sur `d` : la ligne devient rouge et l’invite demande confirmation. Appuyez sur `y` (ou `Enter`) pour supprimer l’entrée de `client.json`, toute autre touche annule. `Local` ne peut pas être supprimé.

## Variables d’environnement

Une nouvelle session hérite d’un ensemble fixe de votre shell — `PATH`, `HOME`, `USER`, `SHELL`, `TERM`, `WT_PROFILE_ID`, `PSMODULEPATH` :

- `--env-inherit <patterns>` — liste séparée par des virgules (ou la variable d’environnement `VTERM_ENV_INHERIT`) ; les jokers `*` sont pris en charge ;
- `-e VAR=VALUE` — injecter des variables supplémentaires (répétable) ;
- `--no-env` — démarrer la session sans aucune variable héritée.

```sh
vtermc -w ~/project -e RUST_LOG=debug --env-inherit "PATH,HOME,TERM"
```

## Toutes les options

```text
-p, --profile [<ID>]          Choisir le profil de terminal (omettre pour une liste interactive)
-w, --working-dir <DIR>       Définir le répertoire de travail
-s, --select                  Choisir serveur et terminal existant interactivement
-a, --add [<SHARE>]           Ajouter un serveur (vtshare1 : chaîne de partage, ou interactif)
    --server <URL>            URL WebSocket du serveur [env : VTERM_SERVER]
-e, --env <VAR=VALUE>         Variables d’environnement supplémentaires
    --no-env                  Ne pas hériter des variables d’environnement
    --env-inherit <PATTERNS>  Motifs de variables d’environnement à hériter [env : VTERM_ENV_INHERIT]
```
