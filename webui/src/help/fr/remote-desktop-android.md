Au-delà des écrans de bureau, le bureau à distance peut diffuser des appareils Android — de vrais téléphones via adb et des émulateurs AVD. Le client les montre dans une fenêtre en forme de téléphone avec saisie tactile, rotation et contrôle à distance du cycle de vie de l’émulateur.

## Prérequis

Le serveur doit trouver la chaîne d’outils Android :

- **SDK Android** (fournit `adb`) — requis pour les appareils comme pour les émulateurs ;
- **Android Studio** — requis pour démarrer les émulateurs AVD.

Les deux sont détectés automatiquement par défaut. Si la détection échoue ou que les outils vivent hors des emplacements standards, réglez `Android SDK directory` / `Android Studio directory` sous `Remote Desktop` → `Screen streaming tools` ; les changements prennent effet immédiatement (appliqués à chaud, sans redémarrage du serveur).

## Sources d’appareils

Les chemins en place, le serveur énumère automatiquement les sources Android disponibles :

- **Appareils réels connectés** — branchez en USB avec le débogage USB activé ; tout ce que `adb devices` voit peut diffuser ;
- **Émulateurs AVD** — les émulateurs en cours apparaissent d’eux-mêmes ; vous pouvez aussi démarrer un AVD choisi à la demande depuis le client (inutile de le lancer à la main d’abord) et l’éteindre à distance.

Chaque source reçoit sa propre entrée dans la barre SCREENS du client.

## Orientation

Le bouton de rotation de la fenêtre d’émulateur règle le `user_rotation` du système via `adb` — la direction d’affichage de l’appareil change, et la fenêtre se réajuste à la nouvelle résolution, s’adaptant aussi bien au portrait qu’au paysage.

## Côté client

Ouvrez l’onglet `Debug` dans le panneau gauche du client de bureau et cliquez le bouton `Android Simulator` dans la barre SCREENS (il n’apparaît que si le serveur expose des sources Android) pour ouvrir la fenêtre d’émulateur ; le sélecteur d’appareil change l’apparence du cadre téléphonique et `Rotate` inverse l’orientation. Voir Utiliser le client.

> L’injection tactile sur appareils réels utilise le canal d’entrée d’adb (Android 4.4+) ; certaines ROM personnalisées par le fabricant la restreignent, auquel cas l’appareil est en visualisation seule.
