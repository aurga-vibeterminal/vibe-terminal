Il y a deux niveaux de personnalisation : patcher la source d'un addon existant, ou en écrire un nouveau sur le modèle d'un intégré et l'importer.

## Patch à chaud de la source

Le bouton crayon sur n'importe quelle carte d'addon ouvre sa source JavaScript pour édition. Enregistrer stocke un instantané qui remplace l'original — pour les addons intégrés comme importés — et le changement prend effet immédiatement, sans redémarrage. Pratique pour ajuster un prompt, corriger une URL ou prototyper une idée sur le système en production.

`Reset` abandonne l'instantané et restaure la source d'origine (intégrés seulement — un addon importé est déjà à vous).

![Éditer la source d'un addon — l'instantané enregistré remplace l'original](images/addon-edit-source.jpg)

## Écrire votre propre addon

Un addon, ce sont deux fichiers :

- `<id>.json` — le manifeste : `id`, `name`, `description`, `author`, `email`, `date`, `type`, et `config_fields` (key, label, type de champ, valeur par défaut, description) que la Web UI transforme en formulaire ;
- `<id>.js` — la source : un simple fichier JavaScript (exécuté par le moteur QuickJS embarqué, avec un pont `addon.get_config()` vers les champs configurés) exportant les fonctions attendues par son type — p. ex. `chat_completion()` pour un fournisseur LLM, listes d'outils et appels au pont pour un addon MCP.

Le démarrage le plus rapide est d'exporter un addon existant : le bouton `Export` de sa carte télécharge un zip avec le manifeste et la source ensemble. Choisissez l'intégré le plus proche de votre objectif (ils vivent aussi dans `crates/vterm-server/src/addon/builtins/`), ajustez l'id, le name, le type et les fonctions exportées, puis installez le résultat avec `Upload Addon`. Les addons importés peuvent être supprimés avec le bouton corbeille.

![Exporter un addon — manifeste et source JavaScript en un seul zip](images/addon-export.jpg)

## Stockage

Les addons importés et les instantanés de source vivent dans le dossier `addons/` du répertoire de configuration sous forme de `<id>.json` + `<id>.js`, et survivent aux redémarrages. Pour un id intégré, seul le remplacement `.js` est honoré — l'intégré garde son propre manifeste.
