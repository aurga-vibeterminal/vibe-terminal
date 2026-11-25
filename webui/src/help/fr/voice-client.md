Pressez `Ctrl` deux fois rapidement (sous ~300 ms) n’importe où dans le client de bureau pour ouvrir la boîte de saisie vocale.

## Enregistrement

La boîte démarre immédiatement l’enregistrement : la forme d’onde liquide suit votre voix, le sélecteur en haut change de micro (le choix est mémorisé), et un minuteur montre le temps écoulé. `Stop` termine le clip et l’envoie en transcription (`Reset` relance).

![La phase d’enregistrement — forme d’onde, sélecteur de micro et minuteur](images/voice-client-record.jpg)

## Traitement

Une animation de flux de particules joue pendant que le serveur transcrit l’audio.

## Édition

La transcription atterrit dans une zone de texte éditable :

- la pastille de mode bascule entre `original` et `polished` ; `Polish` fait tourner le LLM une fois, et `Auto polish` le fait juste après chaque transcription ;
- `Re-record` jette le texte et repart de zéro ; `Insert` réenregistre et insère la nouvelle transcription dans le texte existant au curseur (remplaçant toute sélection), pour étendre ou amender ce que vous avez déjà dicté — le curseur atterrit juste après l’extrait inséré ;
- survolez la zone de texte pour révéler un bouton de copie dans son coin supérieur droit : un clic copie le texte et il se mue en coche ; éditer le texte ou changer de mode le réarme pour une autre copie ;
- `Submit` (ou `Ctrl+Entrée`) tape le texte dans le terminal actif ; `Échap` ferme la boîte.

![La phase d’édition — pastille de mode, polissage auto, la rangée d’actions avec Insert, et le bouton de copie au survol](images/voice-client-editing.jpg)

## Configuration du polissage

`Polish` a besoin d’un LLM de polissage vocal : dans la section `LLM` de l’interface Web, activez `Enable voice text polish` dans la liste par fonction (un fournisseur ordinaire plus cette bascule suffisent). Sans lui le bouton `Polish` reste grisé et l’option de polissage automatique est cachée.
