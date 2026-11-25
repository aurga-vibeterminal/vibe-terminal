Eksplorator plików operuje na systemie plików po stronie serwera. Pasek narzędzi u góry lewego panelu przełącza między Recent / Explorer / Git / Debug / Search; Explorer to przeglądarka plików.

## Przycisk Home: skok do cwd terminala

Kliknij przycisk `Home` (domku) na pasku narzędzi:

- Przełącza na widok `Explorer`;
- Skacze do **katalogu roboczego aktywnego terminala** (jego cwd);
- Przypina ten katalog jako drzewo i czyści pole wyszukiwania — zawsze startujesz od „gdzie terminal, tam patrzysz".

![Eksplorator plików w trybie zwykłym — Home, okruszki i płaska lista](images/files-explorer.jpg)

## Tryb drzewa i wyszukiwanie

- **Przypnij jako drzewo**: przycisk pinezki na prawym końcu okruszków przełącza między płaską listą a trybem drzewa. W trybie drzewa foldery rozwijają się w zwijane drzewo katalogów (skok z Home przypina automatycznie);
- **Wyszukiwanie**: pole u góry filtruje bieżącą listę po nazwie w trakcie pisania; `Search in Folder` z menu kontekstowego Recent uruchamia rekurencyjne szukanie po całym katalogu.

![Eksplorator plików w trybie drzewa — przypięty korzeń rozwija się w zwijane drzewo katalogów](images/files-explorer-tree.jpg)

## Podstawowe operacje

Prawy przycisk na pliku lub folderze:

- **Tworzenie**: `New Folder` / `New File`;
- **Edycja**: `Rename`; `Copy` / `Cut` / `Paste` (działa między katalogami);
- **Usuwanie**: `Delete`, z oknem potwierdzenia;
- **Związek z terminalem**: `Open in Terminal` otwiera nową kartę terminala w tym katalogu; `cd to Directory` robi cd bieżącego terminala tam;
- **Związek z systemem**: `Reveal in File Explorer` pokazuje lokalny katalog w menedżerze plików systemu;
- **Ścieżki**: `Copy Full Path` / `Copy Relative Path`;
- **Publikacja**: `Host with HTTP server` publikuje katalog jako usługę plików HTTP (zob. Publikacja przez HTTP); `Open in App` otwiera go aplikacją agenta wykrytą na serwerze.

Każda operacja działa na systemie plików po stronie serwera; sesje zdalne zachowują się dokładnie jak lokalne.
