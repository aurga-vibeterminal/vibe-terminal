Le bureau à distance vous laisse observer et piloter les écrans du serveur depuis le client de bureau en temps réel : le bureau de la machine est diffusé dans une fenêtre de votre côté, avec votre souris et votre clavier injectés — comme si vous étiez assis devant. Au-delà des écrans de bureau, les émulateurs Android et simulateurs iOS peuvent être consultés et pilotés dans des fenêtres en forme de téléphone.

![Client de bureau — une fenêtre de bureau à distance utilisée aux côtés des terminaux](images/remote-desktop-overview-hero.webm)

## Ce que ça sait faire

- **Écrans de bureau** — chaque écran physique du serveur diffuse dans sa propre fenêtre client redimensionnable ;
- **Appareils / émulateurs Android** — les appareils connectés et émulateurs AVD apparaissent comme sources, consultables et touchables dans une fenêtre en forme de téléphone, avec rotation et démarrage/arrêt distant de l’émulateur ;
- **Simulateur iOS** (serveurs macOS) — consulter le simulateur iOS de Xcode dans une fenêtre en forme de téléphone avec sélecteur d’appareil et rotation.

## Comment ça marche

Le serveur capture l’écran et l’encode avec encodeur, cadence, plafond de résolution et débit configurables ; le client décode dans des fenêtres flottantes et renvoie les événements d’entrée sur la connexion chiffrée client-serveur existante — pas de ports supplémentaires.

La préparation varie selon la plateforme : Windows peut faire tourner le serveur d’écran comme service système et exige un pilote d’affichage virtuel sur les machines headless ; Linux utilise un consentement Portal plus déblocage de session ; macOS requiert la permission d’enregistrement d’écran. Voir Configuration du serveur ; les appareils Android sont couverts dans Appareils Android, et les fenêtres du client dans Utiliser le client.
