Het gebied `CHANGES` van het Git-paneel dekt de dagelijkse commit-lus af: wijzigingen bekijken, stagen, verwerpen, stashen en committen met een bericht.

## Stagen en ontstagen

- Niet-gestagede wijzigingen staan onder `CHANGES`, elk bestand met een statusbadge — `M` gewijzigd, `A` toegevoegd, `D` verwijderd, `?` ontracked, `!` conflict (submodulregels dragen een extra `SUB`-tag);
- Over een regel zweven onthult inline actieknoppen: `+` staget een niet-gestaged bestand, `−` ontstaget een gestaged, en `×` op een niet-gestagede regel verwerpt het; de knoppen `Stage All Changes` / `Unstage All Changes` in de sectiekoppen regelen alles in één keer;
- `Ctrl`-klik wisselt bestanden in een meervoudige selectie en `Shift`-klik kiest een bereik — een actieknop geldt dan voor elk gekozen bestand;
- Op een bestandsnaam klikken opent zijn diff zodat je de wijziging kunt bekijken voordat je beslist;
- Rechtsklik op een bestand biedt `View File History`, `Stash This File`, `Copy Full Path` / `Copy Relative Path`, `Reveal in File Explorer` en een `cd`-snelkoppeling die de terminal naar de map van het bestand springt.

![De sectie STAGED CHANGES — gestagede bestanden met per regel Unstage-knoppen](images/git-staging-staged.jpg)

## Committen

1. Stage wat erin moet;
2. typ een bericht in het commitvak (`Ctrl+Enter` commit) of klik op de AI-knop om er een te genereren (zie LLM-assistentie);
3. klik op `Commit`.

![Het commitvak — berichtinvoer met de AI-genereerknop](images/git-staging-commit.jpg)

Na het committen, zweef over de geschiedeniskop — de balk met de branchnaam boven de commitgeschiedenis — en de knoppen `Fetch`, `Pull` en `Push` faden rechts in. `Push` stuurt de huidige branch meteen naar zijn upstream; voor opties als `Force with lease` gebruik je het pushvenster van de branchlijst (zie Push, checkout & meer).

![Over de geschiedeniskop zweven onthult de Fetch-, Pull- en Push-knoppen](images/git-staging-push.jpg)

## Stashen

- De archiefknop in de kop `CHANGES` opent het venster `Stash All Changes`, waarin een optioneel bericht de stash beschrijft;
- rechtsklik op gekozen niet-gestagede bestanden biedt `Stash This File` (of `Stash <n> Selected Files` voor een meervoudige selectie) om alleen die paden te stashen — submodulregels zijn uitgesloten;
- stashes verzamelen zich onder `STASHES` als `WIP on <branch>`-invoeren; er één uitvouwen somt de gevangen bestanden op, en op een bestand klikken toont zijn diff;
- Over een stashregel zweven toont `Apply` (herstellen en houden), `Pop` (toepassen en verwijderen) en `Drop` (verwijderen zonder toe te passen) — `Pop` en `Drop` vragen eerst om bevestiging, en het rechtsklikmenu voegt `View Diff` toe;
- De sectie `STASHES` blijft verborgen zolang de stashlijst leeg is.

![Het venster Stash All Changes met zijn optionele bericht](images/git-staging-stash-all.jpg)

![De sectie STASHES — een stash uitgevouwen naar zijn bestanden, met Apply-, Pop- en Drop-knoppen](images/git-staging-stashes.jpg)

![Pop past de stash toe en verwijdert hem na een bevestiging uit de lijst](images/git-staging-stash-pop.jpg)

## Verwerpen

`Discard Changes` op een niet-gestaged bestand gooit alle niet-gecommitte wijzigingen erin weg — er is een bevestigingsvenster en de actie **kan niet ongedaan worden gemaakt**.

> Een commit bevat alleen gestagede inhoud; niet-gestagede wijzigingen blijven in de werkboom.
