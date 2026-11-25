Terminals zijn de kern van Vibe Terminal: elk tabblad herbergt één of meer terminalsessies, kan worden gesplitst in deelvensters of naar een eigen venster worden gesleept. Terminals bereiken de lokale server en externe/gedeelde servers via een relay.

## Terminals aanmaken

- Klik op de knop `+` in de titelbalk om een tabblad met het standaardprofiel te openen;
- Klik op de chevron naast `+` voor het profielmenu: terminalprofielen per server gegroepeerd (lokaal, verbonden, gedeeld) — vouw een groep uit en kies een profiel om een terminal op die server te openen;
- Sleep een lokale map naar de titelbalk en laat los om een nieuw tabblad met die werkmap te openen (alleen lokale server; het losgelaten item moet een map zijn).

![Titelbalk — de knop voor een nieuw tabblad en het profielmenu](images/terminal-usage-creation.jpg)

## Muisvastlegging en contextmenu

- Rechtsklik in een terminal opent zijn contextmenu: plakken, deelvensters splitsen, snelle commando's, omgevingsvariabeleverzamelingen, alle tekst kopiëren, MCP-instelopdrachten en meer, afhankelijk van de sessie;
- Toepassingen die de muis vastleggen (opencode, pi, vim, tmux, htop, …) ontvangen de muisgebeurtenissen in plaats daarvan — houd `Shift` ingedrukt om de gewone bewerkingen ook bij zo'n app uit te voeren:
  - **Tekst kopiëren**: houd `Shift` ingedrukt en selecteer de tekst en klik dan — `Shift` nog steeds ingedrukt — ergens anders om de selectie op te heffen; de selectie wordt naar het klembord gekopieerd;
  - **Contextmenu**: `Shift` + rechtsklik of `Ctrl` + rechtsklik opent het contextmenu; een rechtsklik terwijl de linker muisknop wordt ingehouden opent het ook (een bestaande selectie wordt eerst opgeheven);
  - **Links**: `Shift` + klik volgt links zoals gebruikelijk.

## Splitsingen en slepen-organisatie

- **Splitsen**: rechtsklik in een terminal → `Split` → `Left` / `Right` / `Top` / `Bottom` verdeelt het tabblad in deelvensters, elk een onafhankelijke terminal;

  ![Een tabblad splitsen — het rechtskliksplitsmenu en de resulterende deelvensters](images/terminal-usage-split.gif)

- **Samenvoegen door slepen**: sleep een tabblad op een andere terminal en er verschijnen dokzones (links/rechts/boven/onder); laat los om het tabblad als nieuw deelvenster in het doeltabblad samen te voegen;

  ![Samenvoegen door slepen — een tabblad op een andere terminal wordt een nieuw deelvenster](images/terminal-usage-merge.mp4)

- **Losmaken naar een venster**: sleep een tabblad uit de tabstrook (of op een ander venster) en het hele tabblad verlaat zijn venster als een nieuw. Sessies draaien altijd op de server — splitsingen, samenvoegingen en sleepbewerkingen onderbreken nooit een draaiende opdracht.

## Tabbladen en tabbladgroepen

- **Tabblad** — één terminaltabblad, eventueel met meerdere gesplitste deelvensters;
- **Tabbladgroep** — een container die tabbladen organiseert. Standaard volgen groepen de servers: de tabbladen van de lokale server staan in `Ungrouped`, en elke gedeelde server krijgt een groep `Name (Server)`; aangepaste groepen worden ondersteund;
- **Groepen en vensters** — elk venster toont één actieve groep via de groepskeuze in de titelbalk (of `All`); `Move Group to New Window` scheurt een hele groep los naar een eigen venster, zodat je een venster of monitor aan één machine wijdt;
- **Groepen en servers** — een groep onthoudt zijn server, zodat het wisselen van groep de werkruimte van een server herfocusseert; het profielmenu volgt de actieve groep en toont alleen profielen van relevante servers.

![De groepskeuze — groepslijst en verplaatsen naar nieuw venster](images/terminal-usage-groups.jpg)
