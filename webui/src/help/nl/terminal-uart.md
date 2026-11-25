UART-serieel verandert de seriële console van een apparaat (debugpoort, embedded-board, SBC) in een terminaltabblad dat je kunt bekijken en bedienen als elke andere terminal.

## Een poort configureren

In de sectie `UART` van de Web UI → `Add UART Port`:

- `Name` — wordt als titel van de terminal getoond (bijv. "Debug Console");
- `Serial Port` — kies uit de gedetecteerde poorten (het aantal staat bovenaan de pagina);
- `Baud Rate` / `Data Bits` / `Parity` / `Stop Bits` / `Flow Control` — afstemmen op de firmware van het apparaat;
- `Terminal Type` en `Encoding` — de codering is standaard Auto (UTF-8) en kan de firmware volgen;
- `Show Advanced Options` onthult verdere seriële parameters.

Opgeslagen poorten verschijnen in de lijst `Configured UART Ports` en kunnen op elk moment worden bewerkt of verwijderd.

![De UART-instelpagina — het poortformulier](images/terminal-uart-config.jpg)

## Gebruik in de client

Geconfigureerde poorten openen als terminaltabbladen zoals gewone serversessies:

- Seriële gegevens worden getoond zodra ze binnenkomen; toetsenbordinvoer gaat rechtstreeks naar de poort;
- Het rechtsklikmenu biedt `Clear` (scherm legen) en een submenu `Encoding` (codering live wisselen; de huidige is aangevinkt);
- UART-sessies hebben geen shellfuncties (geen Snelle commando's, Apps of Envs-menu-items).

Typische toepassingen: router/switch-consoles, logs van embedded-boards, bewaking van flash-jigs.
