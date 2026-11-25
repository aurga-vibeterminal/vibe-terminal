Les addons étendent le serveur avec de nouvelles capacités sans toucher au cœur : fournisseurs LLM, backends de reconnaissance vocale, mise à jour DNS dynamique, surfaces d'outils MCP supplémentaires et détection des apps d'agents de code vivent tous en addons. La section `Addons` de la Web UI les gère.

## Catégories

Chaque addon déclare un type, et la section regroupe les cartes en conséquence :

- **Applications** — apprennent à l'observateur de terminaux à reconnaître une CLI d'agent de code (Claude Code, Codex, Kimi Code, Pi et OpenCode sont intégrés) : détection de processus, état occupé/inactif, crochets d'automatisation ;
- **LLM Providers** — backends de complétion de chat parmi lesquels les fonctions IA choisissent (un fournisseur compatible OpenAI est intégré) ;
- **Voice / ASR** — backends de reconnaissance vocale (l'addon intégré `vibe-voice` pointe vers le serveur ASR local supervisé) ;
- **Dynamic DNS** — maintiennent un nom d'hôte aligné sur l'adresse actuelle (Cloudflare est intégré) ;
- **MCP** — surfaces d'outils MCP supplémentaires servies sur l'hôte MCP (EasyEDA Pro est intégré) ;
- tout le reste atterrit dans le groupe **Other**.

## Addons intégrés et importés

- Les addons **intégrés** sont livrés dans le programme et toujours disponibles. Leur manifeste et leur source restent d'origine — une source remplacée peut être restaurée à tout moment ;
- Les addons **importés** sont les vôtres : un zip contenant un manifeste et une source JavaScript, installé via le bouton `Upload Addon` ;
- Le badge `Modified` signale un addon dont vous avez remplacé la source.

Chaque carte affiche le type, les badges et une bascule d'activation ; dépliée, elle révèle la description, l'auteur, les champs de configuration et le code source.

![La section Addons — cartes groupées avec badges et bascules d'activation](images/addon-section.jpg)

## Configuration

Le manifeste d'un addon déclare ses champs de configuration, et la carte dépliée en fait un formulaire. Les valeurs sont stockées par addon et ne prennent effet que s'il est activé ; les champs de type mot de passe restent masqués. Désactiver un addon conserve sa configuration — en le réactivant, vous reprenez où vous vous étiez arrêté.
