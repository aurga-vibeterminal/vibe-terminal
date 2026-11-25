Le série UART transforme la console série d’un appareil (port de débogage, carte embarquée, SBC) en un onglet de terminal que vous pouvez observer et manipuler comme n’importe quel autre terminal.

## Configurer un port

Dans la section `UART` de l’interface Web → `Add UART Port` :

- `Name` — affiché comme titre du terminal (p. ex. « Debug Console ») ;
- `Serial Port` — choisissez parmi les ports détectés (le nombre est affiché en haut de la page) ;
- `Baud Rate` / `Data Bits` / `Parity` / `Stop Bits` / `Flow Control` — à faire correspondre au firmware de l’appareil ;
- `Terminal Type` et `Encoding` — l’encodage vaut Auto (UTF-8) par défaut et peut suivre le firmware ;
- `Show Advanced Options` expose d’autres paramètres série.

Les ports enregistrés apparaissent dans la liste `Configured UART Ports` et peuvent être modifiés ou retirés à tout moment.

![La page de réglages UART — le formulaire de port](images/terminal-uart-config.jpg)

## Utilisation dans le client

Les ports configurés s’ouvrent comme des onglets de terminal au même titre que les sessions ordinaires du serveur :

- les données série s’affichent à leur arrivée ; la saisie clavier va directement au port ;
- le menu contextuel offre `Clear` (vider l’écran) et un sous-menu `Encoding` (changer d’encodage à la volée ; l’actuel est coché) ;
- les sessions UART n’ont pas de fonctions shell (pas de Commandes rapides, Apps ni éléments de menu Envs).

Usages typiques : consoles de routeur/commutateur, journaux de cartes embarquées, surveillance de banc de flashage.
