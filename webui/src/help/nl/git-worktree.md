Worktrees checken meerdere branches van één repository uit in afzonderlijke mappen: parallel ontwikkelen zonder de hoofdwerkboom aan te raken, de branch daarna terugmergen en opruimen. Vibe Terminal maakt de hele cyclus klik-en-klaar.

## Een worktree aanmaken

1. Klik in de branchlijst van het Git-paneel met rechts op de doelbranch (lokaal of extern) → `New Worktree`;
2. het venster noemt de nieuwe worktree naar de branch — bewerk naar believen; bevestig, en de server maakt de worktreemap en checkt de branch daar uit;
3. **er opent automatisch een nieuw terminaltabblad** met zijn werkmap op de nieuwe worktree — begin direct geïsoleerd te ontwikkelen terwijl de hoofdwerkboom op zijn branch blijft.

![Het venster New Worktree — de naam is standaard de branch](images/git-worktree-create.jpg)

![Het nieuwe tabblad, geopend voor de pas gemaakte worktree](images/git-worktree-terminal.jpg)

## Parallel werken

Elke worktree is een volledig onafhankelijke werkmap: builds en tests voor de hoofdbranch en de featurebranch draaien zij aan zij in verschillende tabbladen zonder storen. In een worktree uitgecheckte branches dragen in de branchlijst een worktree-markering die hun pad toont, en committen en pushen werken rechtstreeks vanuit het tabblad van de worktree als op elke branch.

Rechtsklik op een worktree-branch voegt twee worktree-exclusieve invoeren toe: `Reveal in File Explorer` (lokale sessies) opent zijn map in de systeembestandsbeheerder, en `Open in Terminal` opent daar nog een tabblad.

## Terugmergen en opruimen

1. Commit en push in het tabblad van de worktree zoals gebruikelijk;
2. schakel het hoofdtabblad naar de doelbranch (bijv. `main`), rechtsklik de featurebranch → `Merge into Current Branch` om het werk over te brengen;

   ![De worktree-branch terug in de hoofdbranch mergen](images/git-worktree-merge.jpg)

3. na het mergen, rechtsklik de branch → `Delete Worktree`: dit verwijdert de worktreemap en zijn branch samen. Zorg eerst dat alles gemerged of gepusht is — het venster waarschuwt opnieuw en de actie kan niet ongedaan.

   ![De Delete Worktree-bevestiging — map en branch worden samen verwijderd](images/git-worktree-delete.jpg)

