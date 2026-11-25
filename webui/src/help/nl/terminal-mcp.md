Elke terminal is een eersteklas burger van MCP (Model Context Protocol): externe agents sturen terminals aan via de MCP-gereedschappen van de server — terminals lijsten, het scherm lezen, opdrachten uitvoeren, op voltooiing wachten — waardoor de codeeragent die je al gebruikt echte interactieve shells kan bedienen.

## De terminal-MCP-gereedschappen van de server

Rond terminalsessies kan een MCP-client:

- `list_terminals` — alle open terminals; `get_terminal` — titel, cwd en appstatus van één terminal;
- `get_screen` — de huidige zichtbare schermtekst; `get_scrollback` — recente geschiedenisregels;
- `send_command` — een opdrachtregel uitvoeren (Enter toegevoegd); `send_input` — rauwe tekst zonder Enter (`y` bevestigen, formulieren invullen); `send_key` — benoemde toetsen (Enter, Ctrl+C, pijlen);
- `clear_terminal` — de buffer legen; `resize_terminal` — de PTy herschalen; `wait_terminal` — blokkeren tot de terminal niet meer bezet is.

## Een terminal adresseren: het Terminal-ID

Gereedschapsaanroepen richten zich op een terminal via zijn Terminal-ID. Rechtsklik in een terminal → `MCP` → `Copy Terminal ID`, en plak het ID vervolgens bij je agent zodat die exact die terminal aanstuurt.

![Terminal-rechtsklik — het MCP-submenu](images/terminal-mcp-menu.jpg)

## Agent-app-registratie met één klik

Het submenu `MCP` somt ook de op de server gedetecteerde agent-apps op (editor-CLI's en dergelijke). Op een app klikken typt een kant-en-klare `mcp add`-opdracht in de terminal — **zonder afsluitende regeleinde**; controleer die en druk zelf op Enter om de MCP-dienst van deze server bij die agent te registreren. Invoeren verbergen zich zolang een app al in de sessie draait, zodat tekst niet in de app-UI belandt.

> Typische lus: kopieer het Terminal-ID naar je agent → `send_command` start een build → `wait_terminal` tot klaar → `get_screen` leest het resultaat. Samen met Automatisch bevestigen (zie het terminalhoofdstuk) draaien lange interactieve taken vrijwel zonder toezicht.
