Les agents de code (Claude Code, Codex, Kimi, Pi, OpenCode…) ne savent que ce qui atteint leur entrée. `Ajouter du contexte` est la façon de leur remettre chemins de fichiers et extraits de code sans retaper : faites glisser des fichiers depuis l’`Explorer`, ou faites glisser du texte sélectionné depuis l’éditeur de code et la vue de diff Git, droit dans le terminal de l’agent ou la saisie de conversation Copilot.

## Faire glisser des fichiers

Pressez un fichier — ou une sélection multiple — dans la liste de fichiers de l’`Explorer` et faites-le glisser :

- **Sur un terminal** : le ou les chemins complets sont tapés au point de dépôt, séparés par des espaces, prêts à faire partie de votre prochain message à l’agent ;
- **Sur la saisie de conversation Copilot** : les chemins sont insérés dans le brouillon de message, un par ligne, pour assembler un message multi-fichiers avant l’envoi.

Les chemins glissés depuis le gestionnaire de fichiers de votre système d’exploitation et déposés sur un terminal sont automatiquement mis entre guillemets, si bien que les noms contenant des espaces survivent intacts.

![Glisser un fichier de la liste de l’Explorer dans un terminal tape son chemin complet](images/terminal-context-drag-file.gif)

## Faire glisser du code sélectionné comme contexte

Sélectionnez du texte dans l’éditeur de code ou dans un diff Git, puis pressez la sélection elle-même et faites-la glisser sur un terminal :

- quand le terminal **exécute un agent de code**, le dépôt insère un *bloc de contexte* — le chemin du fichier, la plage de lignes et l’extrait enveloppé dans une clôture de code :

````text
File: crates/vterm/src/ui/terminal_view.rs
Lines: 558-566
```
fn can_accept_native_drop(&self, info: &DndInfo) -> bool { … }
```
````

- quand le terminal est sur une **invite de shell simple**, le dépôt tape plutôt le texte sélectionné brut, pour que le même geste serve aussi aux commandes ordinaires ;
- quand le diff est consulté à un commit précis, le porte en plus une ligne `Commit:` épinglant la révision exacte.

![Glisser une sélection d’éditeur dans un terminal d’agent dépose un bloc de contexte File/Lines](images/terminal-context-drag-selection.mp4)

## Copier le contexte

Le même bloc vit dans le menu contextuel : `Copy Context` dans l’éditeur de code et la vue de diff le copie dans le presse-papiers. Collez-le dans la saisie de conversation Copilot — ou dans n’importe quel autre agent — quand le glisser est peu pratique. L’animation ci-dessus montre le bloc obtenu dans la saisie de l’agent.

## Astuces

- Sélection multiple dans l’`Explorer` pour déposer plusieurs chemins d’un coup ;
- la saisie de conversation Copilot envoie droit dans le terminal comme une saisie tapée (voir `Historique des agents de code`), donc le contexte ajouté là atteint l’agent exactement comme si vous l’aviez tapé ;
- combinez avec la `Confirmation automatique` pour laisser l’agent agir sur le contexte déposé sans invites supplémentaires.
