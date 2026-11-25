De terminalanalysator aan serverzijde volgt de terminaluitvoer: hij detecteert opdrachtprompts, herkent taakvoltooiing en classificeert met een LLM wat de terminal toont. `Auto Confirm` is de meest gebruikte functie — wanneer de terminal stopt bij een gewone bevestigingsprompt (zoals `Proceed? [Y/n]`), wordt Enter voor je verstuurd, zodat lange taken niet langer vastlopen op vragen.

## Configuratie

De sectie `Terminal Analysis` van de Web UI:

- `Terminal observer` — de hoofdschakelaar van de analyse aan serverzijde;
- `Auto confirm` — stuurt Enter alleen voor prompts die als **gewone bevestigingen** zijn geclassificeerd; andere vragen worden nooit blind beantwoord;
- `Analysis delay (seconds)` — hoe lang te wachten nadat een agent inactief wordt voordat er wordt geanalyseerd, wat overbodige LLM-aanroepen tussen taken voorkomt;
- `Confirmation grace (seconds)` — het venster waarin je de controle kunt overnemen nadat een bevestiging is gedetecteerd;
- `Notify on completion` — zendt een gebeurtenis uit wanneer uitvoer als taak-voltooid wordt geclassificeerd (koppel het aan meldingen).

> De notitie op de pagina: wanneer de automatische modus van je eigen agent aanstaat, kun je `Auto Confirm` in de client beter uitzetten zodat de twee automatiseringen niet over elkaar heen antwoorden.

## LLM-afhankelijkheid

De classificatie wordt door een LLM gedaan: kies een `Provider` en vul de inloggegevens in de sectie `LLM` in (gedeeld door terminalclassificatie, Git-commitberichten, de chatassistent en andere AI-functies). Zonder LLM werkt de analysator nog steeds met verminderde nauwkeurigheid — houd automatisch bevestigen in dat geval uit.

![De instelpagina Terminal Analysis](images/terminal-auto-confirm.jpg)

![De instelpagina LLM — kies een provider en vul inloggegevens in](images/terminal-auto-confirm-llm.jpg)
