Szybkie polecenia przypinają polecenia build/test/deploy projektu do menu kontekstowego terminala — jedno kliknięcie, by uruchomić, dostępne wszędzie. Polecenia są utrzymywane per repozytorium w `.vterm/commands.json` w katalogu głównym repozytorium.

## Tworzenie commands.json

Prawy przycisk w terminalu → `Commands`:

- Gdy repozytorium sesji nie ma jeszcze commands.json, menu oferuje jedną pozycję, `Generate commands.json` — kliknięcie każe serwerowi wykryć typ projektu (npm / Cargo / .NET / Xcode / Gradle-Android / generyczny) i zapisać ostrożny szablon startowy. Istniejący niepusty plik **nigdy nie jest nadpisywany**;
- Położenie pliku: `.vterm/commands.json` w katalogu głównym repozytorium Git (bieżący katalog roboczy dla folderów poza Git).

## Dopracowanie

Wygenerowany szablon zawiera tylko zwykłe polecenia wykrytego typu projektu. Niech Twój agent kodu (np. agent edytora podłączony do MCP tego serwera) przeczyta projekt i dopracuje plik:

```json
{
  "commands": [
    { "name": "Build", "command": "cargo build --release" },
    { "name": "Test verbose", "command": "cargo test -- --nocapture",
      "env": { "RUST_LOG": "debug" } },
    { "name": "Package (Windows)", "command": "./packaging/windows/package.bat",
      "os": "win" }
  ]
}
```

Tablica najwyższego poziomu też jest akceptowana; `env` jest opcjonalne i ustawia zmienne przed uruchomieniem polecenia; stary klucz `value` nadal jest czytany. Opcjonalne pole `os` ogranicza pozycję do systemu operacyjnego, na którym działa `vterm-server`: użyj `"win"` dla Windows, `"linux"` dla Linuksa lub `"osx"` dla macOS. Jeśli `os` zostanie pominięte, ustawione na `"all"` lub będzie miało nierozpoznaną wartość, pozycja jest dostępna na każdej platformie. Serwer obserwuje CWD i zmiany commands.json i rozsyła je — otwórz menu ponownie, a nowe polecenia już są.

## Uruchamianie

Prawy przycisk → `Commands` → kliknij pozycję, aby uruchomić ją w tym terminalu (opcjonalne env jest stosowane najpierw). Menu jest niedostępne, gdy w sesji działa aplikacja agenta, żeby tekst polecenia nie wpadał do jej UI.

## Przykład: build i instalacja Androida przez Wi-Fi

Pełny przewodnik na projekcie Gradle-Android: przypnij konfigurację bezprzewodowego ADB oraz przepływ build-instalacja do menu kontekstowego, a potem przebuduj aplikację jednym kliknięciem.

**1. Przełącz telefon na ADB bezprzewodowy.** Te dwa polecenia wpisuje się raz na urządzenie — `adb tcpip 5555` gdy telefon jest na USB, potem `adb connect` z adresem telefonu:

![Terminal — przełączanie telefonu Android na ADB bezprzewodowy przez adb tcpip 5555 i adb connect](images/terminal-commands-adb-setup.gif)

**2. Pozwól agentowi kodu napisać polecenia.** Otwórz agenta kodu projektu (tu Claude Code) w terminalu i poproś o dodanie poleceń build i instalacji — czyta on układ projektu i wpisuje pozycje do `.vterm/commands.json`:

![Terminal — Claude Code edytuje commands.json z poleceniami build i instalacji APK](images/terminal-commands-agent-edit.mp4)

**3. Buduj i instaluj z menu.** Podziel terminal, by wyjście buildu miało własny panel, a potem prawy przycisk → `Commands` → pozycja — Gradle składa APK i instaluje je na telefonie przez Wi-Fi:

![Terminal — dzielenie panelu i uruchomienie polecenia Android build & install z menu kontekstowego](images/terminal-commands-split-run.mp4)
