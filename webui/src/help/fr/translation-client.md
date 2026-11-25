Une fois le serveur configuré, toutes les surfaces du client proposent la traduction via la même icône de globe. Les résultats sont mis en cache par élément, donc basculer dans un sens comme dans l'autre ne répète jamais une requête.

## Messages du panneau de chat

Chaque en-tête de message porte une icône de traduction — à côté de l'heure sur vos propres messages, à droite du nom de l'expéditeur et de l'heure sur les réponses du modèle, et à droite de l'étiquette `Réflexion...` sur les blocs de raisonnement :

- **Premier clic** : envoie le Markdown brut du message au serveur et échange le corps du message contre sa traduction. L'icône s'allume pendant que la traduction est affichée ;
- **Autre clic** : restaure le texte original ;
- La traduction reste liée au message tant que la conversation reste ouverte, y compris sur les blocs `Réflexion...` une fois dépliés.

Les messages traduits se restituent en Markdown normal, donc listes, tableaux et blocs de code gardent leur forme.

![Un message de chat avec l'icône de traduction dans l'en-tête et le corps traduit dessous](images/translation-chat-message.jpg)

## Brouillon de message de commit

Dans la zone de commit du panneau Git, un bouton de traduction apparaît à côté du bouton IA dès que votre brouillon dépasse une ligne :

- **Cliquez** pour traduire tout le brouillon — le texte traduit remplace le brouillon dans la zone en aperçu en lecture seule ;
- **Cliquez à nouveau** pour revenir à votre formulation d'origine (le brouillon lui-même n'a jamais été modifié) ;
- La saisie est désactivée pendant l'aperçu ; revenez d'abord en arrière, puis continuez à éditer.

![La zone de commit avec le bouton de traduction et l'aperçu traduit](images/translation-commit-draft.jpg)

## Popup de détails du commit

Survolez un commit dans l'historique Git pour ouvrir la popup de détails. À côté du bouton de copie du hachage se trouve une icône de traduction : cliquez pour traduire le message de commit dans la popup ; cliquez à nouveau pour voir l'original. Le message traduit se restitue comme l'original — ligne d'objet, corps Markdown et blocs de code compris.

![La popup de détails du commit — boutons copier et traduire sur la ligne du hachage](images/translation-git-tip.jpg)

## Sélections dans l'éditeur de code

Sélectionnez du texte dans l'éditeur de code — les sélections normales et en bloc (rectangulaires) fonctionnent —, clic droit puis `Traduire` (la première entrée du menu) :

1. Un badge globe plein apparaît sur le numéro de ligne de la première ligne sélectionnée ;
2. Une carte de traduction s'ouvre immédiatement avec un marqueur *Translating…* et se remplit quand le serveur répond ;
3. La carte est collante : elle reste en place quand vous faites défiler, et seul son bouton de fermeture, `Esc` ou un clic à l'extérieur la renvoient. Survoler le badge la fait revenir.

Chaque sélection traduite obtient son propre badge, et traduire à nouveau les mêmes lignes se contente de réafficher la carte en cache. Les badges disparaissent quand vous ouvrez un autre fichier.

![Une sélection dans l'éditeur de code avec Traduire dans le menu contextuel](images/translation-editor-menu.jpg)

## Sélections dans la vue de diff

La vue de diff fonctionne pareil : sélectionnez des lignes du corps du diff, clic droit, `Traduire`. Le badge remplace le nouveau numéro de ligne de la première ligne sélectionnée, et la carte du résultat s'ancre dessous. Les sélections en mode édition se traduisent aussi ; passer à un autre fichier ou commit efface les badges.

![Un diff avec une sélection traduite — badge de gouttière et carte de traduction](images/translation-diff-card.jpg)

## Ce qui est traduit

Le serveur indique au modèle d'où vient le texte :

- **Prose** (messages de chat, messages de commit) — la mise en forme Markdown est préservée et seule la langue naturelle est traduite ;
- **Sélections de fichiers** (éditeur, diff) — le nom du fichier voyage avec la requête, donc le modèle connaît les marqueurs de commentaires du fichier (`//`, `#`, `--`, …). Les marqueurs de commentaires sont retirés et le texte du commentaire est traduit comme de la prose ; le Markdown à l'intérieur du commentaire (titres, listes, blocs de code) survit. Si la sélection est du code ordinaire, seuls les commentaires et les littéraux de chaîne sont traduits — les identifiants et la structure restent intacts.
