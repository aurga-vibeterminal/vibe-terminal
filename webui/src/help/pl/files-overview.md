Zarządzanie plikami pozwala pracować z plikami na serwerze wprost z klienta: lewy pasek aktywności ma dwa wpisy — `Recent` (ostatnie projekty) i eksplorator plików — używane obok kart terminala. Wszystko operuje na serwerze sesji: maszynie lokalnej albo zdalnej/udostępnianej przez przekaźnik.

## Recent: ostatnie projekty

`Recent` wypisuje foldery, które Twoi agenci kodu otworzyli ostatnio na bieżącym serwerze, najnowsze najpierw:

- **Kliknij** pozycję, a eksplorator plików przełącza się na ten katalog;
- **Prawy przycisk** otwiera menu szybkich akcji:
  - `Open in VS Code` — otwórz katalog w VS Code na tej maszynie;
  - `Reveal in File Explorer` — pokaż w menedżerze plików systemu;
  - `Copy Full Path` / `Copy Relative Path` — pełna ścieżka albo względna do repozytorium;
  - `Search in Folder` — szukaj plików w tym katalogu;
  - `Open in Terminal` — otwórz tam nową kartę terminala; `cd Here` — cd obecnego terminala tam;
  - `Host with HTTP server` — opublikuj katalog jako usługę plików HTTP (zob. Publikacja przez HTTP);
  - `Open in App` — otwórz aplikacją agenta wykrytą na serwerze.

![Lewy panel — lista ostatnich projektów](images/files-overview.jpg)

## Eksplorator plików

Karta `Explorer` przegląda i zarządza systemem plików po stronie serwera: nawigacja po katalogach, widok drzewa i płaski, wyszukiwanie, tworzenie/zmiana nazwy/usuwanie i inne operacje, plus przesyłanie przeciąganiem między serwerem a tą maszyną. Zob. Eksplorator plików i Przesyłanie plików.
