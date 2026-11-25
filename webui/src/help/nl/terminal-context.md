Code-agents (Claude Code, Codex, Kimi, Pi, OpenCode…) weten alleen wat hun invoer bereikt. `Context toevoegen` is de manier om ze bestandspaden en codefragmenten zonder overtypen te overhandigen: sleep bestanden vanuit de `Explorer`, of sleep geselecteerde tekst vanuit de code-editor en de Git-diffweergave, rechtstreeks naar de terminal van de agent of de Copilot-chatinvoer.

## Bestanden naar binnen slepen

Druk op een bestand — of een meervoudige selectie — in de bestandslijst van de `Explorer` en sleep het:

- **Naar een terminal**: de volledige paden worden op de loslaatplek getypt, spatiegescheiden, klaar om deel te worden van je volgende prompt aan de agent;
- **Naar de Copilot-chatinvoer**: de paden worden in het berichtenconcept ingevoegd, één per regel, zodat je een meerbestanden-prompt kunt samenstellen voordat je verstuurt.

Paden die vanuit de bestandsbeheerder van je besturingssysteem naar een terminal worden gesleept, worden automatisch aangehaald, zodat namen met spaties heel blijven.

![Een bestand uit de Explorer-bestandslijst naar een terminal slepen typt het volledige pad](images/terminal-context-drag-file.gif)

## Geselecteerde code als context slepen

Selecteer tekst in de code-editor of in een Git-diff en druk op de selectie zelf en sleep die naar een terminal:

- Als de terminal **een code-agent draait**, voegt het loslaten een *contextblok* in — het bestandspad, het regelsbereik en het fragment in een codehek gewikkeld:

````text
File: crates/vterm/src/ui/terminal_view.rs
Lines: 558-566
```
fn can_accept_native_drop(&self, info: &DndInfo) -> bool { … }
```
````

- Staat de terminal op een **gewone shellprompt**, dan typt het loslaten in plaats daarvan de rauwe geselecteerde tekst, zodat hetzelfde gebaar ook voor gewone opdrachten werkt;
- Wordt de diff op een specifieke commit bekeken, dan draagt het blok bovendien een `Commit:`-regel die de exacte revisie vastpint.

![Een editorselectie naar een agent-terminal slepen laat een File/Lines-contextblok vallen](images/terminal-context-drag-selection.mp4)

## Context kopiëren

Hetzelfde blok leeft in het rechtsklikmenu: `Copy Context` in de code-editor en de diffweergave kopieert het naar het klembord. Plak het in de Copilot-chatinvoer — of bij elke andere agent — als slepen onhandig is. De animatie hierboven toont het resulterende blok in de invoer van de agent.

## Tips

- Meervoudig selecteren in de `Explorer` om meerdere paden tegelijk te laten vallen;
- de Copilot-chatinvoer wordt als getypte invoer rechtstreeks naar de terminal gestuurd (zie `Geschiedenis code-agent`), dus daar toegevoegde context bereikt de agent exact alsof je hem had getypt;
- combineer met `Automatisch bevestigen` om de agent op de gevallen context te laten handelen zonder verdere prompts.
