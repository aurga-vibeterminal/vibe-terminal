Interactieve rebase maakt van `git rebase -i` een visuele stroom: sleep om de commitwachtrij te herschikken, commits één voor één toepassen, conflicten visueel oplossen — geen commandoregel nodig.

## Starten

Open het branchpaneel en typ in het vak `Filter branches...` om de doelbranch te vinden, en rechtsklik erop → `Rebase Current Branch onto '{branch}'` om de huidige branch op het doel te rebasen. Het paneel schakelt naar de rebase-weergave, die vier fasen doorloopt:

![Branch zoeken in het paneel, daarna de rebase op het doel starten](images/git-rebase-search.jpg)

## 1. De commitwachtrij bewerken

De commits op jouw branch die het doel mist vormen een wachtrij:

- **Sleep** om ze te herschikken;
- elke commit heeft een actiedropdown: `pick` (houden) / `squash` (in de vorige vouwen, berichten samenvoegen) / `fixup` (invouwen, bericht verwerpen) / `edit` (stoppen om aan te passen) / `drop` (verwerpen);
- gesquashte en gedropte commits vouwen visueel dicht.

![De rebase-commitwachtrij — herschikken door slepen, acties per commit](images/git-rebase-queue.jpg)

## 2. Toepassen

`Start Rebase` speelt de wachtrij commit voor commit op het doel af met een live voortangsbalk; zonder conflicten loopt hij rechttdoor naar klaar. `Abort` stapt op elk moment uit.

## 3. Conflicten (indien aanwezig)

Een conflict tijdens het toepassen schakelt naar de conflictweergave — een bestandslijst plus de merge-editor met zijn vier modi (`Merge` / `Split` / `Raw` / `AI`), waar elke conflicterende hunk zijn eigen beslissing krijgt (`Ours` / `Theirs` / `Both` …; zie Conflicten oplossen). Zodra elk bestand is opgelost, hervat `Continue` de rebase.

![De conflictweergave — merge-beslissingen per hunk](images/git-rebase-resolve-conflict.jpg)

## 4. Klaar

Bij voltooiing bevestigt het paneel `Branch '{branch}' was rebased onto '{target}'` en toont de push-kaart — de rebase herschreef de geschiedenis, dus de externe branch moet geforceerd worden bijgewerkt:

- `Force Push with Lease` — aanbevolen: overschrijft alleen als niemand anders heeft gepusht en gebruikt nooit een gewone `--force`;
- `Not now` — pushen overslaan; de lokale rebase is al voltooid.

![De klaar-pagina — Force Push with Lease of pushen overslaan](images/git-rebase-done.jpg)

> De sessie blijft bruikbaar tijdens de rebase; het paneel sluiten onderbreekt een lopende niet.
