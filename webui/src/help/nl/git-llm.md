De intelligentie van het Git-paneel is LLM-aangedreven: commitberichten met één klik, AI-ondersteund diff-lezen en conflict-oplossing door een agent, hunk voor hunk. Alles berust op een LLM-provider die in de Web UI is geconfigureerd.

## De provider configureren

Open de sectie `LLM` van de Web UI:

- Kies een `Provider` en vul zijn inloggegevens in (de velden worden door het manifest van de provider gedefinieerd); de configuratie is globaal — terminalclassificatie, Git-commitberichten, de chatassistent en andere AI-functies delen dezelfde provider;
- Providers zijn addons van het type LLM: gangbare zijn ingebouwd, en aangepaste providers kunnen als zip in de sectie `Addons` worden geüpload (het manifest declareert de configuratievelden die de Web UI tot een formulier maakt).

![De LLM-sectie van de Web UI — kies een provider en vul inloggegevens in](images/git-llm-provider-config.jpg)

## LLM-functies in Git

- **Commitberichtgeneratie** — klik op de AI-knop naast het commitvak; de LLM leest de gestagede diff en schrijft een bericht, met een animatie in het vak getypt. Het bericht blijft bewerkbaar — commit pas als je tevreden bent;
- **Diff-analyse** — de glitterknop in de kop van de diffweergave analyseert de huidige diff;
- **Conflict-AI-agent** — de AI-modus van de merge-editor van de rebase lost hunks één voor één op met uitleg en betrouwbaarheid (zie Conflicten oplossen).

![Het door AI gegenereerde commitbericht, in het commitvak getypt](images/git-llm-commit-message.jpg)

> De LLM leest repository-inhoud alleen om tekst te produceren; commits en merges gebeuren altijd wanneer jij ze bevestigt.
