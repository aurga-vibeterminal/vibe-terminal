Każdy terminal jest pełnoprawnym obywatelem MCP (Model Context Protocol): zewnętrzni agenci prowadzą terminale przez narzędzia MCP serwera — listować terminale, czytać ekran, wykonywać polecenia, czekać na ukończenie — pozwalając agentowi kodowania, którego już używasz, operować na prawdziwych interaktywnych powłokach.

## Narzędzia MCP terminala serwera

Wokół sesji terminala klient MCP może:

- `list_terminals` — wszystkie otwarte terminale; `get_terminal` — tytuł, cwd i status aplikacji jednego terminala;
- `get_screen` — aktualny widoczny tekst ekranu; `get_scrollback` — ostatnie wiersze historii;
- `send_command` — wykonać wiersz polecenia (z dołączonym Enterem); `send_input` — surowy tekst bez Enter (potwierdzenie `y`, wypełnianie formularzy); `send_key` — nazwane klawisze (Enter, Ctrl+C, strzałki);
- `clear_terminal` — wyczyścić bufor; `resize_terminal` — przeskalować PTY; `wait_terminal` — blokować, dopóki terminal nie będzie wolny.

## Adresowanie terminala: Terminal ID

Wywołania narzędzi celują w terminal przez jego Terminal ID. Prawy przycisk w terminalu → `MCP` → `Copy Terminal ID`, a potem wklej ID agentowi, żeby prowadził dokładnie ten terminal.

![Prawy przycisk terminala — podmenu MCP](images/terminal-mcp-menu.jpg)

## Rejestracja aplikacji agenta jednym kliknięciem

Podmenu `MCP` wypisuje też aplikacje agentów wykryte na serwerze (CLI edytorów i podobne). Kliknięcie aplikacji wpisuje gotowe polecenie `mcp add` do terminala — **bez końcowego nowego wiersza**; przejrzyj je i sam naciśnij Enter, aby zarejestrować usługę MCP tego serwera u tego agenta. Pozycje chowają się, gdy aplikacja już działa w sesji, by tekst nie wpadał do jej UI.

> Typowa pętla: skopiuj Terminal ID agentowi → `send_command` startuje build → `wait_terminal` aż skończy → `get_screen` czyta wynik. Razem z Automatycznym potwierdzaniem (zobacz rozdział o terminalach) długie interaktywne zadania działają niemal bez nadzoru.
