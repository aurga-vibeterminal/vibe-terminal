Gdy uruchamiasz agenta kodu (Claude Code, Codex, Kimi, Pi, OpenCode…) w terminalu, jego rozmowa jest uwięziona w tym oknie terminala — przewijana między wyjściami poleceń, tracona gdy ekran płynie dalej. `Historia rozmów agenta kodu` wynosi całą rozmowę do dedykowanego panelu: każda wymiana pokazana strukturalnie, dawne rozmowy przełączalne, wiadomości tłumaczalne — a z panelu można wysyłać polecenia prosto do terminala.

## Otwarcie panelu

Kliknij przycisk czatu (`AI`) po prawej stronie paska tytułu, aby przełączyć panel Copilot. Rozwijane menu u góry wypisuje **wszystkie dawne rozmowy** agenta działającego w aktywnej sesji terminala, najnowsze najpierw; wybór jednej ją ładuje.

![Panel Copilot — rozwijane menu rozmów i strumień wiadomości](images/terminal-agent-history.jpg)

## Jak to działa

- Serwer wie, która aplikacja agenta działa w terminalu, i parsuje jej **natywne pliki sesji** (format JSONL/historii każdego agenta pod jego katalogiem projektu); zmiany pliku wpływają na żywo, utrzymując widok rozmowy aktualnym;
- Wspierani agenci: `claude` / `codex` / `kimi` / `pi` / `opencode`; inne aplikacje nie mają jeszcze panelu rozmów;
- Wiadomości niosą rolę (Ty / agent) i typ (tekst / rozumowanie **Thinking**), wraz z nazwą modelu i czasem, renderowane jako Markdown.

## Funkcje panelu

- **Przełączanie rozmów**: rozwijane menu `Conversations` u góry porusza się między dawnymi sesjami; każda pozycja pokazuje skrót i czas;
- **Tłumaczenie**: każdą wiadomość agenta można na żądanie przetłumaczyć; tłumaczenie zastępuje tekst w miejscu (i da się je przełączyć z powrotem);
- **Bezpośrednie wysyłanie poleceń**: wpisz w polu na dole (Shift+Enter dla nowego wiersza) i naciśnij Enter — polecenie wędruje prosto do bieżącego terminala, jak wpisywanie we własne pole agenta bez zajmowania widoku terminala.

## Współgranie z innymi funkcjami

- Panel i terminal to dwa widoki tej samej sesji: pytania agenta w terminalu są tam nadal odpowiadane (albo przez Automatyczne potwierdzanie); panel służy do przeglądu historii, nie do przechwytywania interakcji;
- W połączeniu z `Terminal MCP` (poprzednia strona) zewnętrzni agenci mogą też czytać ekran terminala i wykonywać polecenia — rozmowa pokazana tu plus akcje MCP tworzą pełny zapis pracy agenta.
