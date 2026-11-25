`vtermc` to klient wiersza poleceń: łączy się z serwerem, tworzy lub podłącza sesje terminali i zarządza zapisaną listą serwerów — wszystko w zwykłym terminalu, przez SSH albo na maszynie headless. Współdzieli `~/.vibeterminal/client.json` z klientem desktopowym, więc oba widzą te same serwery.

## Uruchamianie terminala

Uruchom `vtermc` bez flag trybu, aby utworzyć nowy terminal na serwerze lokalnym (domyślnie `wss://localhost:5153`) z wykrytą powłoką i strumieniować go do bieżącego terminala. Wpisz `exit` (lub `Ctrl+D`) w zdalnej powłoce, aby zakończyć sesję.

Sesje żyją na serwerze: jeśli połączenie padnie — utrata sieci, zamknięcie okna, ubicie `vtermc` — sesja dalej działa i można ją później podłączyć przez `vtermc -s`.

- `vtermc -p <id>` — start z konkretnym profilem terminala (lista profili jest w sekcji `Profiles` Web UI);
- `vtermc -p` — bez id profilu: selektor interaktywnie wypisuje profile serwera;
- `vtermc -w <dir>` — ustawia katalog roboczy nowej sesji.

Gdy stdin nie jest terminalem (potok lub plik), `vtermc` przechodzi w tryb wsadowy: stdin trafia do sesji, a wyjście płynie aż do jej zamknięcia — przydatne ze skryptów lub agentów:

```sh
echo "make test" | vtermc
```

## Łączenie z serwerem

- `--server wss://host:port` — połączenie z konkretnym serwerem zamiast lokalnego (adresy IPv6 w nawiasach: `wss://[fd00::1]:5153`); zmienna środowiskowa `VTERM_SERVER` ustawia tę samą wartość domyślną;
- Uścisk dłoni jest odpowiadany tajnym kluczem serwera: serwery dodane przez `-a`/`-s` niosą swój klucz w `client.json`; serwer lokalny wraca do zmiennej środowiskowej `TERMINAL_GUI_SECRET_KEY` (lub klucza w jawnym tekście w `server.json`).

## Dodawanie serwerów

`-a` zapisuje serwer w `client.json` i kończy — nigdy nie otwiera terminala. Dwie drogi:

- `vtermc -a` — interaktywne pytania: `Server name`, `Server address` i `Secret key`. Adres musi mieć postać `wss://host:port` (błędne wejście jest ponownie żądane; pusty adres anuluje). Pusta nazwa wraca do adresu;
- `vtermc -a "vtshare1:..."` — wklej zaszyfrowany ciąg udostępniania. Kliknięcie `QR Code` parowania w sekcji `Server` Web UI kopiuje go do schowka; `vtermc` odszyfrowuje go i automatycznie wypełnia nazwę, adres i klucz tajny.

Detale warte poznania:

- ponowne dodanie istniejącego adresu aktualizuje ten wpis w miejscu (zachowuje jego id) zamiast go duplikować;
- adresy lokalne są odrzucane — `Local` jest zawsze niejawnie dostępny i nigdy nie wymaga wpisu;
- wpisy pokazuje też Server Manager klienta desktopowego.

## Wybór serwerów i sesji

`vtermc -s` otwiera dwa pełnoekranowe selektory (wymaga interaktywnego terminala):

1. **Lista serwerów** — najpierw `Local`, potem każdy zarządzany serwer jako `Name  (adres)`. Nawigacja strzałkami lub `j`/`k`, skok cyfrą, `Enter` wybiera, `q` anuluje;
2. **Lista terminali** — najpierw `New Terminal`, potem aktywne sesje tego serwera z katalogiem roboczym, tytułem i serwerem właścicielem. Podłącz się do istniejącej sesji albo utwórz nową (która respektuje `-p` / `-w` jak zwykłe uruchomienie `vtermc`).

Podłączanie nigdy nie przerywa sesji — dalej działa na serwerze między podłączeniami.

### Usuwanie serwera

Na liście serwerów przejdź na zarządzany serwer i naciśnij `d`: wiersz czerwienieje, a podpowiedź prosi o potwierdzenie. Naciśnij `y` (lub `Enter`), aby usunąć wpis z `client.json`, każdy inny klawisz przerywa. `Local` nie może być usunięty.

## Zmienne środowiskowe

Nowa sesja dziedziczy stały zestaw z Twojej powłoki — `PATH`, `HOME`, `USER`, `SHELL`, `TERM`, `WT_PROFILE_ID`, `PSMODULEPATH`:

- `--env-inherit <patterns>` — lista rozdzielona przecinkami (lub zmienna środowiskowa `VTERM_ENV_INHERIT`); obsługiwane są symbole wieloznaczne `*`;
- `-e VAR=VALUE` — wstrzyknięcie dodatkowych zmiennych (powtarzalne);
- `--no-env` — rozpoczęcie sesji bez żadnych dziedziczonych zmiennych.

```sh
vtermc -w ~/project -e RUST_LOG=debug --env-inherit "PATH,HOME,TERM"
```

## Wszystkie opcje

```text
-p, --profile [<ID>]          Wybór profilu terminala (pomiń dla listy interaktywnej)
-w, --working-dir <DIR>       Ustawienie katalogu roboczego
-s, --select                  Interaktywny wybór serwera i istniejącego terminala
-a, --add [<SHARE>]           Dodanie serwera (vtshare1: ciąg udostępniania, lub interaktywnie)
    --server <URL>            Adres WebSocket serwera [env: VTERM_SERVER]
-e, --env <VAR=VALUE>         Dodatkowe zmienne środowiskowe
    --no-env                  Bez dziedziczenia zmiennych środowiskowych
    --env-inherit <PATTERNS>  Wzorce dziedziczonych zmiennych środowiskowych [env: VTERM_ENV_INHERIT]
```
