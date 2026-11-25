Captures d’écran et terminal se rejoignent en un seul enchaînement : un unique raccourci global saisit n’importe quelle zone du bureau, vous l’annotez, la copiez ou l’enregistrez — puis collez l’image du presse-papiers directement dans un terminal, où elle atterrit sous forme de chemin de fichier que le programme en cours (un agent de code compris) peut ouvrir et utiliser. Les deux moitiés vivent dans le client de bureau.

## Prendre une capture d’écran

- Pressez `Shift`+`Alt`+`A` n’importe où : le raccourci est global, il se déclenche quelle que soit l’application qui a le focus tant que le client de bureau tourne (sur macOS, pressez `Shift`+`Command`+`A`) ;
- Chaque écran s’assombrit sous une surcouche de capture. Glissez pour dessiner la sélection — elle peut s’étendre sur plusieurs écrans —, puis glissez à l’intérieur pour la déplacer, ou tirez ses poignées pour la redimensionner ; un badge au-dessus de la sélection donne sa taille en pixels ;
- Pressez `Esc` à tout moment pour annuler.

![La surcouche de capture — une sélection avec son badge de taille et la barre d’annotation](images/terminal-images-capture.jpg)

## Annoter

La barre sous la sélection porte à gauche les outils d’annotation (rectangle et ellipse), au centre six couleurs, et à droite trois boutons d’action — annuler (`✕`), enregistrer (flèche de téléchargement) et copier (`✓`).

- Choisissez un outil et une couleur, puis glissez à l’intérieur de la sélection pour dessiner ; les traits sont rognés à la sélection ;
- `Ctrl`+`Z` annule la dernière annotation, `Ctrl`+`R` la rétablit ;
- Glisser une nouvelle zone hors de la sélection actuelle la rejette — annotations comprises — et en commence une autre.

## Copier ou enregistrer

- `Copier` (`✓`) place la capture annotée dans le presse-papiers — prête à être collée dans un terminal (ci-dessous), une discussion ou toute autre application ;
- `Enregistrer` (flèche de téléchargement) écrit un PNG via une boîte de dialogue ; le nom de fichier par défaut est `vterm-screenshot-<timestamp>.png`.

## Coller une image dans un terminal

- Copiez une image dans le presse-papiers — depuis l’outil de capture ci-dessus, un navigateur ou la capture propre du système d’exploitation ;
- Clic droit sur le terminal et choisissez `Paste Image` ; l’entrée n’apparaît que lorsque le presse-papiers contient réellement une image ;
- L’image est téléversée vers le serveur de la session et y est stockée comme fichier temporaire ; son chemin entre guillemets est ensuite inséré au curseur du terminal exactement comme si vous l’aviez tapé — enveloppé de marqueurs de collage entre crochets quand l’application en cours les gère, afin que shells, éditeurs et agents le reçoivent proprement ;
- Pour le programme, ce n’est que du texte : un agent de code tel que Claude Code ou opencode ouvre l’image depuis le chemin, et les commandes shell ordinaires peuvent l’utiliser comme argument.

![Clic droit du terminal — l’entrée Paste Image](images/terminal-images-paste.jpg)

## Remarques

- Le chemin inséré vit sur le serveur de la session : pour un serveur distant ou WSL, le fichier temporaire est créé sur cette machine, là où les programmes lancés dans le terminal peuvent l’atteindre ;
- Combiné à `Ajouter du contexte aux agents`, c’est le moyen le plus rapide de montrer à un agent ce qui se trouve sur votre écran — capturez, collez, et l’agent tient l’image comme fichier ;
- Sous Linux, la capture fonctionne sous X11 ; les sessions Wayland natives ne gèrent ni le raccourci global ni la surcouche interactive, et si une autre application X11 a déjà saisi `Shift`+`Alt`+`A`, le raccourci ne peut pas s’enregistrer.
