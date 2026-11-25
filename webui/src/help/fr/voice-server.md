La saisie vocale transforme la parole en entrée de terminal : parlez au lieu de taper, avec polissage LLM facultatif de la transcription avant l’envoi. La reconnaissance tourne comme serveur ASR local avec des modèles sur l’appareil — l’audio et le texte restent sur votre machine. Cette page couvre le côté serveur ; l’expérience de bureau vit dans Utiliser le client.

## La carte Local ASR Server

Ouvrez la section `Voice` de l’interface Web et activez la reconnaissance vocale. La carte `Local ASR Server` gère le processus vterm-voice supervisé :

- `Load on Startup` — démarrer le serveur ASR en même temps que le backend ;
- `Bind Address` (`127.0.0.1` localhost seul / `0.0.0.0` toutes interfaces) et `Port` (défaut `8398`) — où écoute le point de terminaison ASR ;
- `Model Directory` + `Local Model` — choisissez dans le catalogue intégré : familles Qwen3-ASR, Fun-ASR et Parakeet, chacune en variantes full / 8 bits / 4 bits. `Download` récupère le modèle depuis Hugging Face ou ModelScope dans le répertoire avec une barre de progression en direct ; des commandes `git clone` manuelles sont affichées en secours, et les modèles déjà sur disque sont marqués disponibles ;
- `Device` — le backend d’inférence (`Auto`, `CPU`, `CUDA`, `Vulkan`, `Metal`), tel que rapporté par le serveur ;
- `Access Token` — secret partagé du point de terminaison ; le bouton dé en génère un ;
- `Save` persiste la carte (les modifications restent un brouillon local jusque-là), et `Start Server` / `Stop Server` contrôlent le processus. L’icône d’état dans le titre de la carte reflète la joignabilité et l’état du modèle (chargé / en chargement / erreur).

![La section Voice — la carte Local ASR Server avec modèle, appareil et jeton](images/voice-server-local-asr.jpg)

> **Contention CUDA** : quand un autre programme d’inférence CUDA occupe déjà le GPU (p. ex. llama.cpp tournant un modèle local), laisser vterm-voice utiliser `CUDA` aussi peut ralentir drastiquement les deux. Dans ce cas réglez `Device` sur `CPU` — ces modèles ASR compacts sont assez rapides sur CPU.

## Fournisseur et test

Le menu déroulant des fournisseurs liste les addons ASR — l’addon `vibe-voice` intégré pointe vers ce serveur supervisé, donc le choisir est la façon pour le panneau d’utiliser vos modèles locaux.

La zone `Test` essaie tout sans client : `Record` (micro → forme d’onde → transcription) et `Live` (reconnaissance en flux), chacun avec minutages et bouton de copie.

![La zone Test — les onglets Record et Live](images/voice-server-test.jpg)
