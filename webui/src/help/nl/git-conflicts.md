Wanneer een rebase of merge op conflicten stuit, schakelt het paneel naar de conflictweergave: links de conflictbestandslijst, rechts de merge-editor. Conflicterende bestanden verschijnen ook in de lijst `CHANGES` met een `!`-badge — er op één klikken springt direct deze weergave in in plaats van een gewone diff. De editor biedt vier oplossingsmodi en je kunt er op elk moment tussen wisselen.

![De conflictweergave — links de bestandslijst, rechts de merge-editor](images/git-conflicts-view.jpg)

## De conflictbestandslijst

Bestanden worden naar status gegroepeerd: de automatisch met succes gemergede (automatisch gestaged) en de nog handmatige aandacht behoevende. Klik op een bestand om het in de editor te openen; de lijst werkt live bij terwijl je oplost, tot hij leeg is.

## De vier oplossingsmodi

- **Merge** (standaard) — conflicten worden hunk voor hunk gepresenteerd; kies per hunk `take ours` / `take theirs` of bewerk het gemergede resultaat inline;
- **Split** — een diff wordt in fijnere regelniveau-segmenten gebroken, zodat beide zijden stukje voor stukje binnen één hunk kunnen worden overgenomen;
- **Raw** — gewoon bewerken met `diff3`-conflictmarkeringen (`<<<<<<<` / `=======` / `>>>>>>>`) voor handafstelling;
- **AI** — een agent lost het bestand hunk voor hunk op: de LLM analyseert elke conflicterende hunk en produceert oplossingskaarten met uitleg en betrouwbaarheid. Je kunt instructies typen om de strategie te sturen (bijv. "voorkeur voor theirs, behoud de log-regels"), pauzeren, in een andere modus fijnafstemmen en hervatten — al opgeloste hunks worden overgeslagen.

In elke modus wordt de oplossing in het bestand van je sessie geschreven en gestaged, exact als een handmatige bewerking.

## De rebase voortzetten

Zodra elk conflict is opgelost, gaat de rebase door naar de volgende commit; je kunt ook op elk punt afbreken om naar de pre-rebase-toestand terug te keren.
