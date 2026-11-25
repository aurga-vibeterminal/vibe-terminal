Chaque terminal est un citoyen de première classe du MCP (Model Context Protocol) : des agents externes pilotent les terminaux via les outils MCP du serveur — lister les terminaux, lire l’écran, exécuter des commandes, attendre la fin — permettant à l’agent de code que vous utilisez déjà de manipuler de vrais shells interactifs.

## Les outils MCP de terminal du serveur

Autour des sessions de terminal, un client MCP peut :

- `list_terminals` — tous les terminaux ouverts ; `get_terminal` — titre, cwd et état d’application d’un terminal ;
- `get_screen` — le texte visible actuel de l’écran ; `get_scrollback` — les lignes d’historique récentes ;
- `send_command` — exécuter une ligne de commande (Entrée ajoutée) ; `send_input` — texte brut sans Entrée (confirmer `y`, remplir des formulaires) ; `send_key` — touches nommées (Entrée, Ctrl+C, flèches) ;
- `clear_terminal` — vider le tampon ; `resize_terminal` — redimensionner le PTY ; `wait_terminal` — bloquer jusqu’à ce que le terminal ne soit plus occupé.

## Adresser un terminal : le Terminal ID

Les appels d’outils ciblent un terminal par son Terminal ID. Clic droit dans un terminal → `MCP` → `Copy Terminal ID`, puis collez l’identifiant à votre agent pour qu’il pilote exactement ce terminal.

![Clic droit terminal — le sous-menu MCP](images/terminal-mcp-menu.jpg)

## Enregistrement d’application d’agent en un clic

Le sous-menu `MCP` liste aussi les applications d’agent détectées sur le serveur (CLI d’éditeur et similaires). Cliquer une application tape une commande `mcp add` prête à l’emploi dans le terminal — **sans saut de ligne final** ; vérifiez-la et appuyez vous-même sur Entrée pour enregistrer le service MCP de ce serveur auprès de cet agent. Les entrées se cachent pendant qu’une application tourne déjà dans la session, pour que le texte n’atterrisse pas dans son interface.

> Boucle typique : copier le Terminal ID vers votre agent → `send_command` lance un build → `wait_terminal` jusqu’à la fin → `get_screen` lit le résultat. Combiné à la Confirmation automatique (voir le chapitre terminaux), les longues tâches interactives tournent quasiment sans surveillance.
