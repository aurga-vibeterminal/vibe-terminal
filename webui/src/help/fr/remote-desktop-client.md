Le client de bureau ouvre les écrans distants depuis la barre SCREENS en haut du panneau Debug. Chaque écran vit dans sa propre fenêtre flottante utilisable aux côtés des terminaux.

## Ouvrir la barre SCREENS

1. Connectez-vous au serveur cible (directement ou via un relais) ;
2. cliquez l’onglet `Debug` dans la barre d’activités de gauche (l’icône outils) ;
3. la barre `SCREENS` apparaît en haut du panneau : un bouton d’affichage par écran de bureau physique du serveur, suivis des bascules `iOS Simulator` et `Android Simulator`.

![Le panneau Debug — la barre SCREENS avec les boutons d'affichage et les bascules de simulateur](images/remote-desktop-client-screens.jpg)

## Fenêtres d’écran de bureau

Cliquez un bouton d’affichage pour ouvrir la fenêtre de cet écran :

- la fenêtre est librement redimensionnable (jusqu’à 800×600) et l’image se réorganise avec elle ;
- les serveurs multi-écrans reçoivent une fenêtre par écran ; plusieurs peuvent être ouvertes à la fois ;
- pendant la connexion, la fenêtre montre un espace réservé « préparation du bureau » ; si le serveur manque de permission de capture, la fenêtre affiche directement les instructions de permission de la plateforme ;
- recliquez le bouton de la barre ou fermez la fenêtre pour arrêter le visionnage (l’état actif du bouton reste synchronisé avec la fenêtre).

## Fenêtre de simulateur iOS

Cliquez `iOS Simulator` pour ouvrir une fenêtre à cadre téléphonique de taille fixe (264×640 portrait) :

- le sélecteur d’appareil en haut change l’apparence du téléphone ; la fenêtre se réajuste à la résolution choisie ;
- `Rotate` inverse portrait/paysage (fenêtre 524×380 en paysage) ;
- requiert un serveur macOS faisant tourner le simulateur iOS de Xcode.

## Fenêtre d’émulateur Android

`Android Simulator` fonctionne pareil : cadre téléphonique fixe, sélecteur d’appareil, bouton de rotation. Les sources diffusable incluent les émulateurs AVD en cours et les appareils réels connectés ; un AVD peut aussi être démarré ou éteint à la demande depuis le client (voir Appareils Android).

> Les fenêtres diffusant un serveur localhost sont automatiquement exclues de la capture d’écran pour éviter un miroir de rétroaction ; les fenêtres diffusant une machine distante restent capturables pour qu’un enregistrement extérieur les montre encore.
