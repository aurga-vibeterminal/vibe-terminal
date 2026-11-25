La traduction transforme le texte que vous rencontrez dans le client dans la langue de votre choix : réponses du chat IA, messages de commit, commentaires de code, ou toute sélection dans l'éditeur de code ou un diff. C'est un modèle de traduction configuré sur le serveur qui fait le travail — le client se contente d'envoyer le texte, de restituer le résultat et de le mettre en cache pour que vous puissiez basculer entre l'original et la traduction.

## Où la traduction est disponible

- **Panneau Copilot** — chaque en-tête de message (les vôtres, les réponses du modèle et les blocs `Réflexion...`) porte une icône de traduction. Un clic remplace le corps du message par sa traduction ; un autre clic restaure l'original ;
- **Brouillon de message de commit** — la zone de commit du panneau Git affiche un bouton de traduction dès que votre brouillon s'étend sur plusieurs lignes. Il traduit le brouillon entier en place, et un clic de plus ramène votre formulation d'origine ;
- **Popup de détails du commit** — survoler un commit dans l'historique Git ouvre une popup de détails ; l'icône de traduction à côté du hachage passe le message de commit dans votre langue ;
- **Éditeur de code & vue de diff** — sélectionnez n'importe quoi (les sélections en bloc comptent aussi), clic droit puis `Traduire`. Un petit badge globe marque les lignes traduites dans la gouttière, et une carte avec la traduction s'ouvre à côté de la sélection.

## Comment ça marche

1. Le client envoie le texte au serveur — avec le nom du fichier quand le texte provient de l'éditeur ou d'un diff, pour que le modèle sache quelle syntaxe de commentaires le fichier utilise ;
2. Le LLM de traduction du serveur le traduit dans la langue cible configurée dans l'interface Web. La mise en forme Markdown (titres, listes, tableaux, blocs de code) est préservée ; dans les sélections de fichiers, les marqueurs de commentaires comme `//` ou `#` sont reconnus comme décoratifs et retirés, tandis que le code et les identifiants restent intacts ;
3. Le client restitue le résultat en Markdown là d'où venait la requête — en place pour les messages de chat, ou dans une carte défilante pour les sélections de l'éditeur et des diffs.

## Ce qu'il vous faut

- Un serveur avec la fonction de traduction activée — la configuration unique est décrite dans *Configuration du serveur* (section `LLM` de l'interface Web) ;
- Un client de bureau connecté à ce serveur. Rien ne se configure côté client : toutes les surfaces reprennent automatiquement les mêmes réglages du serveur.

Si la traduction n'est pas configurée, le client affiche l'erreur du serveur (par exemple dans la carte de traduction) au lieu d'un résultat.
