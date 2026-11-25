Het Git-paneel bouwt versiebeheer in de client in: stagen, committen, branches beheren, worktrees, interactieve rebase, conflictoplossing en pushen — zonder je terminalvenster te verlaten. Alles werkt op de Git-repository van de huidige terminalsessie: in welke map de sessie ook draait, dat is de repository die het paneel toont.

## Openen

Klik op het tabblad `Git` in de linkeractiviteitenbalk (repositorygegevens verschijnen alleen als de actieve sessie binnen een Git-repository zit). De repository van een tabblad volgt zijn actieve sessie; het kruimelspoor van de repository laat je tussen ouder- en kindrepositories springen, en submodules worden apart vermeld.

![Het Git-paneel — wijzigingen, geschiedenis en branches](images/git-overview-panel.jpg)

## Wat het kan

- **Stagen & committen** — per bestand of bulk stage/ontstage, wijzigingen verwerpen, committen met een bericht, door AI gegenereerde commitberichten (zie LLM-assistentie);
- **Geschiedenis** — commitgeschiedenis en per bestand, tussen revisies vergelijken en herstellen;
- **Branches** — lokale/externe branches en tags uitchecken, aanmaken en verwijderen;
- **Worktrees** — een branch uitchecken in een geïsoleerde werkboom (er opent automatisch een nieuw terminaltabblad), en bij klaar zijn terugmergen (zie Worktrees);
- **Interactieve rebase** — de commitwachtrij visueel herschikken/squashen/bewerken met conflictoplossing (zie Interactieve rebase);
- **Conflictoplossing** — een merge-editor met vier modi, waaronder een AI-agent die hunks één voor één oplost (zie Conflicten oplossen);
- **Push & overig** — een volwaardig pushvenster, stashes, omgang met detached-HEAD (zie Push, checkout & meer).

De volgende pagina's behandelen elk gebied in die volgorde.
