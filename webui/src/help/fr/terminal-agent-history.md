Quand vous faites tourner un agent de code (Claude Code, Codex, Kimi, Pi, OpenCode…) dans un terminal, sa conversation est piégée dans cette fenêtre de terminal — défilée parmi les sorties de commandes, perdue dès que l’écran avance. L’`Historique de conversation des agents de code` hisse toute la conversation dans un panneau dédié : chaque échange affiché structuré, les conversations passées commutables, les messages traduisibles — et vous pouvez envoyer des commandes au terminal directement depuis le panneau.

## Ouvrir le panneau

Cliquez sur le bouton de conversation (`AI`) à droite de la barre de titre pour basculer le panneau Copilot. Le menu déroulant en haut liste **toutes les conversations passées** de l’agent tournant dans la session de terminal active, plus récentes d’abord ; en choisir une la charge.

![Le panneau Copilot — le menu déroulant des conversations et le flux de messages](images/terminal-agent-history.jpg)

## Comment ça marche

- Le serveur sait quelle application d’agent tourne dans le terminal et analyse ses **fichiers de session natifs** (le format JSONL/historique de chaque agent sous son répertoire de projet) ; les changements du fichier arrivent en flux, gardant la vue de conversation à jour ;
- Agents pris en charge : `claude` / `codex` / `kimi` / `pi` / `opencode` ; les autres applications n’ont pas encore de panneau de conversation ;
- Les messages portent un rôle (vous / agent) et un type (texte / raisonnement **Thinking**), avec le nom du modèle et l’heure, rendus en Markdown.

## Fonctions du panneau

- **Changer de conversation** : le menu déroulant `Conversations` en haut navigue entre les sessions passées, chaque entrée affiche un résumé et son heure ;
- **Traduire** : n’importe quel message d’agent peut être traduit à la demande ; la traduction remplace le texte en place (et peut être basculée en retour) ;
- **Envoyer des commandes directement** : tapez dans la boîte du bas (Shift+Entrée pour un saut de ligne) et appuyez sur Entrée — la commande va directement dans le terminal courant, comme taper dans la propre boîte de saisie de l’agent sans occuper la vue du terminal.

## Articulation avec les autres fonctions

- Le panneau et le terminal sont deux vues de la même session : les questions de l’agent dans le terminal y sont toujours répondues (ou via la Confirmation automatique) ; le panneau sert à relire l’historique, pas à intercepter l’interaction ;
- Combiné au `Terminal MCP` (page précédente), des agents externes peuvent aussi lire l’écran du terminal et exécuter des commandes — la conversation affichée ici plus les actions MCP forment l’enregistrement complet du travail de l’agent.
