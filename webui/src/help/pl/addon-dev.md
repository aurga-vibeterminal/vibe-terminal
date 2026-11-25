Są dwa poziomy dostosowania: załatać źródło istniejącego dodatku albo napisać nowy na wzór wbudowanego i wgrać go.

## Hot-patch źródła

Przycisk ołówka na każdej karcie dodatku otwiera jego źródło JavaScript do edycji. Zapisanie tworzy migawkę, która nadpisuje oryginał — dla wbudowanych i wgranych dodatków — a zmiana działa natychmiast, bez restartu. Sprawdza się przy poprawianiu promptu, naprawie URL czy prototypowaniu pomysły na żywo.

`Reset` porzuca migawkę i przywraca pierwotne źródło (tylko wbudowane — wgrany dodatek jest już twój).

![Edycja źródła dodatku — zapisana migawka nadpisuje oryginał](images/addon-edit-source.jpg)

## Pisanie własnego dodatku

Dodatek to dwa pliki:

- `<id>.json` — manifest: `id`, `name`, `description`, `author`, `email`, `date`, `type` oraz `config_fields` (key, label, typ pola, wartość domyślna, opis), z których Web UI robi formularz;
- `<id>.js` — źródło: zwykły plik JavaScript (uruchamiany przez wbudowany silnik QuickJS, z mostem `addon.get_config()` do skonfigurowanych pól), eksportujący funkcje, które wywołuje jego typ — np. `chat_completion()` dla dostawcy LLM, listy narzędzi i wywołania mostu dla dodatku MCP.

Najszybszy start to eksport istniejącego dodatku: przycisk `Export` na jego karcie pobiera zip z manifestem i źródłem razem. Wybór wbudowanego dodatku najbliższego celowi (leżą też w `crates/vterm-server/src/addon/builtins/`), zmiana id, name, type i eksportowanych funkcji, po czym instalacja wyniku przez `Upload Addon`. Wgrane dodatki można usunąć przyciskiem kosza.

![Eksport dodatku — manifest i źródło JavaScript w jednym zipie](images/addon-export.jpg)

## Przechowywanie

Wgrane dodatki i migawki źródeł leżą w folderze `addons/` katalogu konfiguracji jako `<id>.json` + `<id>.js`, więc przeżywają restarty. Dla wbudowanego id uwzględniane jest tylko nadpisanie `.js` — wbudowany dodatek zachowuje własny manifest.
