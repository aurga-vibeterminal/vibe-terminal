Les commandes rapides épinglent les commandes build/test/deploy d’un projet dans le menu contextuel du terminal — un clic pour exécuter, disponible partout. Les commandes sont maintenues par dépôt dans `.vterm/commands.json` à la racine du dépôt.

## Créer commands.json

Clic droit dans un terminal → `Commands` :

- quand le dépôt de la session n’a pas encore de commands.json, le menu propose une seule entrée, `Generate commands.json` — un clic fait détecter au serveur le type de projet (npm / Cargo / .NET / Xcode / Gradle-Android / générique) et écrire un modèle de départ prudent. Un fichier existant non vide n’est **jamais écrasé** ;
- emplacement du fichier : `.vterm/commands.json` à la racine du dépôt Git (le répertoire de travail courant pour les dossiers hors Git).

## Affiner

Le modèle généré ne contient que les commandes usuelles du type de projet détecté. Laissez votre agent de code (par exemple un agent d’éditeur connecté au MCP de ce serveur) lire le projet et affiner le fichier :

```json
{
  "commands": [
    { "name": "Build", "command": "cargo build --release" },
    { "name": "Test verbose", "command": "cargo test -- --nocapture",
      "env": { "RUST_LOG": "debug" } },
    { "name": "Package (Windows)", "command": "./packaging/windows/package.bat",
      "os": "win" }
  ]
}
```

Un tableau de premier niveau est aussi accepté ; `env` est facultatif et définit des variables avant l’exécution de la commande ; l’ancienne clé `value` est encore lue. Le champ facultatif `os` limite une entrée au système d’exploitation sur lequel tourne `vterm-server` : utilisez `"win"` pour Windows, `"linux"` pour Linux ou `"osx"` pour macOS. Si `os` est omis, défini à `"all"` ou porteur d’une valeur non reconnue, l’entrée est disponible sur toutes les plateformes. Le serveur surveille le CWD et les changements de commands.json et les diffuse — rouvrez le menu et les nouvelles commandes y sont.

## Exécuter

Clic droit → `Commands` → cliquez une entrée pour l’exécuter dans ce terminal (l’éventuel env est appliqué d’abord). Le menu est indisponible pendant qu’une application d’agent tourne dans la session, pour que le texte de commande ne puisse pas atterrir dans l’interface de l’application.

## Exemple : build et installation Android en Wi-Fi

Une démonstration complète sur un projet Gradle-Android : épingler la configuration ADB sans fil et le flux de build-installation dans le menu contextuel, puis reconstruire l’app en un clic.

**1. Passer le téléphone en ADB sans fil.** Ces deux commandes sont saisies une fois par appareil — `adb tcpip 5555` pendant que le téléphone est en USB, puis `adb connect` avec l’adresse du téléphone :

![Terminal — passage d’un téléphone Android en ADB sans fil avec adb tcpip 5555 et adb connect](images/terminal-commands-adb-setup.gif)

**2. Laisser l’agent de code écrire les commandes.** Ouvrez l’agent de code du projet (ici Claude Code) dans un terminal et demandez-lui d’ajouter les commandes de build et d’installation — il lit la structure du projet et écrit les entrées dans `.vterm/commands.json` :

![Terminal — Claude Code édite commands.json avec les commandes de build et d’installation de l’APK](images/terminal-commands-agent-edit.mp4)

**3. Construire et installer depuis le menu.** Divisez le terminal pour que la sortie de build ait son propre volet, puis clic droit → `Commands` → l’entrée — Gradle assemble l’APK et l’installe sur le téléphone en Wi-Fi :

![Terminal — diviser un volet et exécuter la commande Android de build et d’installation depuis le menu contextuel](images/terminal-commands-split-run.mp4)
