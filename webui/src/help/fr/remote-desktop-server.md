Tout vit dans la section `Remote Desktop` de l’interface Web ; la page montre les réglages qui s’appliquent à la plateforme du serveur.

## Interrupteur principal

`Enable Remote Desktop` — une fois activé, les clients distants connectés peuvent voir le bureau de cette machine et envoyer des entrées.

## Réglages par plateforme

**Windows**

- `Windows Service` — enregistre le serveur d’écran comme service système à démarrage automatique pour que le bureau à distance survive aux redémarrages sans session connectée. `Install` / `Uninstall` déclenche une invite UAC sur le serveur (le bouton affiche la progression pendant l’attente) ; le badge d’état distingue en cours / installé / non installé.
- `Machines headless (sans moniteur)` — sans écran branché (ou réveillé), Windows ne garde aucun bureau capturable. La page lie le téléchargement du pilote open source Virtual Display Driver (ARM64 / x64 choisi automatiquement) ; installez-le pour qu’un bureau virtuel existe toujours.

![Interface Web — la section Remote Desktop sous Windows (cartes service et headless)](images/remote-desktop-server-windows.jpg)

**Linux**

- `Unlock on Connect` — déverrouille automatiquement la session verrouillée au démarrage d’un flux (quand l’environnement le permet) ;
- la capture passe par le Portal ; le `Reset Permission` de la carte `Screen Recording Permission` efface un consentement Portal enregistré pour choisir un autre écran ;
- `Auto-start on Login` (dans la section `Serveur`) — sur les hôtes dotés d’un gestionnaire d’affichage, le serveur démarre avec la session graphique, la capture hérite donc de tout l’environnement de bureau ; sur les hôtes headless, il démarre au boot avant toute connexion. Pour atteindre le bureau quand personne ne s’est connecté localement, activez aussi la connexion automatique dans le gestionnaire d’affichage (GDM, SDDM, …) ; par sécurité, le verrouillage automatique de l’écran peut rester activé — la session et le serveur continuent de fonctionner pendant le verrouillage, et `Unlock on Connect` (ou `loginctl unlock-session`) ouvre la session à distance.
![Interface Web — la carte Screen Recording Permission (Linux)](images/remote-desktop-server-permission.jpg)


**macOS**

- `Screen Recording Permission` — le système exige une approbation explicite pour la capture. Cliquez `Grant Permission` pour ouvrir la boîte système ; `Reset Permission` récupère d’un état bloqué.

## Outils de diffusion d’écran (prise en charge Android)

`Android SDK directory` et `Android Studio directory` localisent adb et l’émulateur ; laissez vides pour la détection automatique. Une fois définis, la page affiche le chemin résolu et sa provenance (auto / env / override) ; `Reset to Auto-Detect` restaure le défaut. Voir Appareils Android pour ce qu’ils activent.

## Encodage du flux

- `Encoder` — Auto ou un moteur précis ; le serveur liste les encodeurs utilisables et grise les indisponibles avec la raison ;
- `Frame rate` — 60 / 30 / 15 i/s ;
- `Max resolution` — plafond côté capture (4K / 2.5K / 2K) ;
- `Bitrate` — `Auto` s’adapte en surface à la résolution (débit de base jusqu’à 1080p, ~4× en 4K, plafonné à 40 Mbps) ; une valeur fixe s’applique immédiatement aux flux en cours, tandis que encodeur / i/s / résolution s’appliquent aux flux démarrés ensuite ;
- le pied de carte montre l’encodeur / les i/s / la résolution / le débit actuellement en vigueur.

![Interface Web — réglages d’encodage du flux](images/remote-desktop-server-stream.jpg)

Cliquez `Save` pour appliquer les changements.
