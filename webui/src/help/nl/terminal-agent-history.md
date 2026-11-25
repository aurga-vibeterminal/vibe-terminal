Als je een code-agent (Claude Code, Codex, Kimi, Pi, OpenCode…) in een terminal draait, blijft het gesprek gevangen in dat terminalvenster — weggeschoven tussen opdrachtuitvoer, verdwenen zodra het scherm verder beweegt. De `Gespreksgeschiedenis van code-agents` tilt het volledige gesprek naar een eigen paneel: elke uitwisseling gestructureerd getoond, eerdere gesprekken wisselbaar, berichten vertaalbaar — en je kunt vanuit het paneel rechtstreeks opdrachten naar de terminal sturen.

## Het paneel openen

Klik op de chatknop (`AI`) rechts in de titelbalk om het Copilot-paneel te wisselen. De dropdown bovenaan somt **alle eerdere gesprekken** op van de agent die in de actieve terminalsessie draait, nieuwste eerst; er één kiezen laadt hem.

![Het Copilot-paneel — de gespreksdropdown en de berichtenstroom](images/terminal-agent-history.jpg)

## Hoe het werkt

- De server weet welke agent-app in de terminal draait en ontleedt de **natieve sessiebestanden** ervan (het JSONL-/geschiedenisformaat van elke agent onder zijn projectmap); wijzigingen aan het bestand streamen live binnen en houden de gespreksweergave actueel;
- Ondersteunde agents: `claude` / `codex` / `kimi` / `pi` / `opencode`; andere apps hebben nog geen gesprekspaneel;
- Berichten dragen een rol (jij / agent) en een type (tekst / **Thinking**-redenering), samen met modelnaam en tijd, gerenderd als Markdown.

## Panelfuncties

- **Gesprekken wisselen**: de dropdown `Conversations` bovenaan beweegt tussen eerdere sessies; elke invoer toont een samenvatting en het tijdstip;
- **Vertalen**: elk agentbericht kan op verzoek worden vertaald; de vertaling vervangt de tekst ter plaatse (en kan worden teruggezette);
- **Direct opdrachten sturen**: typ in het vak onderin (Shift+Enter voor een regeleinde) en druk op Enter — de opdracht gaat rechtstreeks de huidige terminal in, zoals typen in de eigen invoerdoos van de agent zonder de terminalweergave te bezetten.

## Samenspel met andere functies

- Het paneel en de terminal zijn twee weergaven van dezelfde sessie: de vragen van de agent in de terminal worden daar nog steeds beantwoord (of via Automatisch bevestigen); het paneel is voor het doornemen van geschiedenis, niet voor het onderscheppen van interactie;
- Gecombineerd met `Terminal MCP` (vorige pagina) kunnen externe agents ook het terminalscherm lezen en opdrachten uitvoeren — het hier getoonde gesprek plus MCP-acties vormen het volledige verslag van het agentwerk.
