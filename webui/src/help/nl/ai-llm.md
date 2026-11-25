De sectie `LLM` van de Web UI wijst aan elke AI-functie een LLM-provider toe — terminalclassificatie, Git-commitberichten, conflictoplossing, spraaktekst-bijschaving en vertaling. De providers zelf zijn addons van het type LLM, beheerd in de sectie `Addons`.

## Providers zijn addons

LLM-providers leven in de groep `LLM` van de sectie `Addons`:

- Een **OpenAI Compatible LLM**-provider is ingebouwd; aangepaste providers kunnen als zip worden geüpload (knop `Upload`). Het addon-manifest declareert de configuratievelden (API-sleutel, basis-URL, model, …) die de Web UI tot een formulier maakt;
- een provider verschijnt alleen in de functiekiezers zolang zijn addon **ingeschakeld** is — zet hem eerst aan in de Addons-sectie;
- de potloodknop op een addokaart bewerkt zijn bron; een gewijzigde ingebouwde addon kan met `Reset` worden teruggerold.

![De Addons-sectie — LLM-providers zijn addons die ingeschakeld, bewerkt of geüpload kunnen worden](images/ai-llm-addons.jpg)

## Configuratie per functie

Elke functiekaart in de sectie `LLM` heeft zijn eigen inschakelschakelaar en eigen providerkeuze, zodat verschillende taken verschillende modellen kunnen gebruiken:

- **Terminal Classification** — classificeert de terminaltoestand wanneer ingebouwde patronen niet volstaan. `Max Consecutive Failures` schakelt de LLM voor de sessie uit na zo veel opeenvolgende mislukkingen (minimum 3);
- **Git Commit Messages** — genereert commitberichten uit gestagede wijzigingen;
- **Git Conflict Resolve** — lost rebase/merge-conflicten hunk voor hunk op;
- **Voice Text Polish** — schaaft spraakinvoer-transcripties bij voor verzending;
- **Translation** — vertaalt tekst naar de gekozen doeltaal.

Vouw een kaart uit om zijn `Provider` te kiezen en de velden van de provider in te vullen. De configuratie wordt ook bewaard terwijl de functie uitgeschakeld is en geldt pas eenmaal ingeschakeld. Als meerdere functies dezelfde provider-addon gebruiken, klonen de kopieer/plak-knoppen naast `Provider Configuration` diens veldwaarden tussen hen (zie hieronder).

Terminal- en Git-functies bieden een uitvoertaal met een `Auto`-optie (het model beslist); vertaling gebruikt een expliciete English/简体中文-doellijst.

![De LLM-sectie — elke functie kiest zijn eigen provider en inloggegevens](images/ai-llm-features.jpg)

## Kopieer/plak tussen functies

Elke functie bewaart zijn eigen kopie van de providervelden, dus een provider voor meerdere functies gebruiken betekent normaal dezelfde basis-URL, API-sleutel en model opnieuw in elke kaart intypen. De kopregel van `Provider Configuration` heeft twee kleine knoppen die de herhaling wegnemen:

- **Kopiëren** — kopieert de veldwaarden die momenteel in deze kaart worden weergegeven, samen met de provider-addon waartoe ze behoren;
- **Plakken** — verschijnt in een kaart alleen als de daar gekozen provider dezelfde addon is als de gekopieerde configuratie; een klik vult de velden van de kaart met de gekopieerde waarden.

Om één endpoint te delen tussen, zeg, Git-commitberichten en vertaling: kies in de eerste kaart de provider en vul de velden in, druk op **Kopiëren**, open de andere kaart (met dezelfde provider geselecteerd) en druk op **Plakken**. De geplakte waarden worden opgeslagen in de eigen configuratie van die functie, als elke andere veldbewerking, en bewaard daarmee. Het klembord is alleen binnen de pagina — het wordt gewist als je de LLM-sectie verlaat.

![Kopieer- en plakknoppen in de kopregel van Provider Configuration — de velden van een provider dupliceren tussen functies met dezelfde addon](images/ai-llm-duplicate-config.jpg)

## De OpenAI-compatibele provider

De ingebouwde provider werkt met elke OpenAI-compatibele chat-API. Belangrijkste velden:

- `Base URL` / `API Key` / `Model` — endpoint, inloggegevens, modelnaam;
- `Extra Body` — een JSON-object dat in elke verzoek-body wordt samengevoegd, voor modelspecifieke parameters;
- `Max Input Tokens` / `Max Output Tokens` — promptbudget en voltooiingsplafond;
- `Timeout` — time-out per verzoek in seconden; verhoog voor trage lokale modellen.

## Aanbevolen modellen

Deze AI-taken zijn kleine, latentiegevoelige prompts — een snel "precies goed"-model verslaat een groot redeneermodel:

- **Lokaal: Qwen3.5 9B** — de sweet spot voor Git-commitberichtgeneratie, terminalclassificatie, vertaling en spraakbijschaving. Serveer hem met een OpenAI-compatibel endpoint (Ollama, vLLM, llama.cpp) en richt `Base URL` erop;
- **API: DeepSeek V4 Flash (`deepseek-v4-flash`)** — snel genoeg voor interactief gebruik en in staat alle bovenstaande functies aan.

> Denkmodus voegt seconden latentie toe en helpt deze korte taken zelden — zet het in `Extra Body` uit voor een betere ervaring, bijv. `{"enable_thinking": false}` voor Qwen of `{"thinking": {"type": "disabled"}}` voor DeepSeek (de exacte sleutel hangt af van je serving-stack; de JSON wordt woordelijk in de verzoek-body samengevoegd).
