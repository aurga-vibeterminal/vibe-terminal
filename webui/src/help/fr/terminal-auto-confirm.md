L’analyseur de terminal côté serveur surveille la sortie du terminal : il détecte les invites de commandes, reconnaît la fin des tâches et, avec un LLM, classifie ce que le terminal affiche. `Auto Confirm` est sa fonction la plus utilisée — quand le terminal s’arrête sur une confirmation ordinaire (comme `Proceed? [Y/n]`), Entrée est envoyée pour vous, si bien que les longues tâches ne calent plus sur des questions.

## Configuration

La section `Terminal Analysis` de l’interface Web :

- `Terminal observer` — l’interrupteur principal de l’analyse côté serveur ;
- `Auto confirm` — n’envoie Entrée que pour les invites classées **confirmations ordinaires** ; les autres questions ne sont jamais répondues à l’aveugle ;
- `Analysis delay (seconds)` — le délai d’attente après qu’un agent devient inactif avant l’analyse, évitant les appels LLM redondants entre tâches ;
- `Confirmation grace (seconds)` — la fenêtre pendant laquelle vous pouvez reprendre la main après qu’une confirmation est détectée ;
- `Notify on completion` — émet un événement quand une sortie est classée « tâche terminée » (à brancher sur des notifications).

> La note de la page : quand le mode automatique de votre propre agent est actif, préférez désactiver `Auto Confirm` côté client pour que les deux automatisations ne se répondent pas l’une sur l’autre.

## Dépendance au LLM

La classification est faite par un LLM : choisissez un `Provider` et remplissez ses identifiants dans la section `LLM` (partagés par la classification de terminal, les messages de commit Git, l’assistant de conversation et d’autres fonctions IA). Sans LLM, l’analyseur fonctionne encore avec une précision réduite — gardez la confirmation automatique désactivée dans ce cas.

![La page de réglages Terminal Analysis](images/terminal-auto-confirm.jpg)

![La page de réglages LLM — choisir un fournisseur et saisir les identifiants](images/terminal-auto-confirm-llm.jpg)
