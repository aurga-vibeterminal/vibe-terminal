L’intelligence du panneau Git est pilotée par LLM : messages de commit en un clic, lecture de diff assistée par IA et résolution de conflits par agent, hunk par hunk. Tout repose sur un fournisseur LLM configuré dans l’interface Web.

## Configurer le fournisseur

Ouvrez la section `LLM` de l’interface Web :

- choisissez un `Provider` et remplissez ses identifiants (les champs sont définis par le manifeste du fournisseur) ; la configuration est globale — classification de terminal, messages de commit Git, l’assistant de conversation et d’autres fonctions IA partagent le même fournisseur ;
- les fournisseurs sont des addons de type LLM : les courants sont intégrés, et des fournisseurs personnalisés peuvent être téléversés en zip dans la section `Addons` (le manifeste déclare les champs de configuration que l’interface Web transforme en formulaire).

![La section LLM de l’interface Web — choisir un fournisseur et remplir ses identifiants](images/git-llm-provider-config.jpg)

## Fonctions LLM dans Git

- **Génération de message de commit** — cliquez le bouton IA à côté de la boîte de commit ; le LLM lit le diff stagé et écrit un message, tapé dans la boîte avec une animation. Le message reste éditable — ne committez que lorsqu’il vous convient ;
- **Analyse de diff** — le bouton étincelle dans l’en-tête de la vue de diff analyse le diff courant ;
- **Agent IA de conflits** — le mode IA de l’éditeur de fusion du rebase résout les hunks un à un avec explications et confiance (voir Résoudre les conflits).

![Le message de commit généré par IA tapé dans la boîte de commit](images/git-llm-commit-message.jpg)

> Le LLM ne lit le contenu du dépôt que pour produire du texte ; les commits et fusions n’arrivent que lorsque vous les confirmez.
