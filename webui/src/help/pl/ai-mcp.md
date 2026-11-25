Serwer ma wbudowany host Model Context Protocol, a sekcja `MCP` w Web UI go konfiguruje: agenty kodu (Claude Code, Codex, Kimi Code, Pi, OpenCode, Qwen Code, DeepSeek Harness, …) mogą połączyć się z własnego czatu i pracować bezpośrednio na twojej maszynie — czytać i sterować terminalami, a nawet obsługiwać EasyEDA Pro.

## Ustawienia serwera

Włącz serwer MCP i skonfiguruj:

- `Port` (domyślnie `6180`) i `Bind Address` — `127.0.0.1` ogranicza endpoint do localhost; `0.0.0.0` otwiera go na sieć;
- `Access Token` — wymagany dla żądań spoza localhost: zdalni wywołujący muszą przedstawić go jako Bearer token, a żądania z localhost są zawsze wpuszczane. Nigdy nie jest pusty — świeża instalacja dostaje losowy token; kostka generuje nowy, przycisk kopiowania wydaje go;
- `Stream Throttle` — minimalny odstęp między wysyłkami strumienia ekranu do połączonych agentów.

Zmiany działają po `Save`; serwer przeładowuje się bez restartu.

## Endpointy

Każdy zestaw możliwości jest serwowany pod własnym adresem `http://localhost:{port}/{id}/mcp`:

- **Terminal MCP** (`/terminal/mcp`) — wbudowana powierzchnia terminala. Agenty mogą listować terminale, czytać widoczny ekran i przewijanie, wpisywać surowy tekst i nazwy klawiszy, zmieniać rozmiar i czekać na zachętę. Wywołania narzędzi celują w terminal po jego Terminal ID — kliknij terminal prawym przyciskiem → `MCP` → `Copy Terminal ID` i wklej ID do kontekstu agenta (zob. Terminal MCP w sekcji Terminals);
- **EasyEDA Pro MCP** (`/easyeda/mcp`) — wbudowany addon MCP mostkujący EasyEDA Pro do agentów: mały stały zestaw narzędzi plus przeszukiwalny katalog wywołań `eda.*`, przekazywany rozszerzeniu-konektorowi EasyEDA po WebSocket. Najpierw włącz addon w sekcji `Addons` — jego endpoint pojawi się na liście po instalacji, a plakietka pokazuje, czy demon addonu jest połączony. Pole `Timeout` addonu ogranicza każde zmostkowane żądanie (zwiększ dla wolnych operacji EDA);
- **Inne addony MCP** — każdy zainstalowany addon typu MCP dodaje swój endpoint tak samo, każdy z plakietką połączono/rozłączono.

Wybierz CLI agenta z listy rozwijanej nad listą; przycisk kopiowania obok endpointu wyda gotową do uruchomienia komendę konfiguracji.

## Podłączenie agenta

Tokena nigdy nie wkleja się ręcznie. Serwer utrzymuje automatycznie generowany profil środowiskowy `Vibe Terminal MCP Bearer Token` (zob. sekcję Environment Variables), eksportujący `VTERM_MCP_TOKEN`; zastosuj go na terminalu przed podłączeniem agenta, a potem użyj skopiowanej komendy:

- **Aplikacje z komendą powłoki** (Claude Code, Codex) — uruchom skopiowaną komendę `mcp add` w zwykłym terminalu. Codex czyta token ze zmiennej `VTERM_MCP_TOKEN` przez `--bearer-token-env-var`, więc żadna tajemnica nie trafia do wiersza poleceń;
- **Aplikacje konwersacyjne** (Kimi Code, Pi, OpenCode, Qwen Code, DeepSeek Harness) — uruchom aplikację z zastosowanym profilem i wyślij skopiowany prompt w jej rozmowie. Agent bezpośrednio zaktualizuje konfigurację MCP. Kimi używa `bearerTokenEnvVar`; Pi, OpenCode i Qwen Code używają tokenu Bearer z `VTERM_MCP_TOKEN`, więc sam token nigdy nie pojawia się w czacie.

W Pi i Qwen Code `/mcp` przegląda połączenia.

Karta `Command Templates` pod ustawieniami trzyma komendy konfiguracji per aplikacja: edytuj wpis, nadpisz wbudowany albo dodaj szablony dla nowych CLI — podgląd pokazuje dokładnie to, co wyda przycisk kopiowania.
