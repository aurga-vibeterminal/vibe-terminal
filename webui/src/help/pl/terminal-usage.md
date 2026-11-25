Terminale są sercem Vibe Terminal: każda karta mieści jedną lub więcej sesji terminala, może być dzielona na panele albo przeciągnięta do własnego okna. Terminale docierają do serwera lokalnego oraz do zdalnych i udostępnianych serwerów przez przekaźnik.

## Tworzenie terminali

- Kliknij przycisk `+` na pasku tytułu, aby otworzyć kartę z domyślnym profilem;
- Kliknij daszek obok `+`, aby otworzyć menu profili: profile terminali pogrupowane według serwerów (lokalny, połączone, udostępniane) — rozwiń grupę i wybierz profil, aby otworzyć terminal na tym serwerze;
- Przeciągnij lokalny folder na pasek tytułu i upuść, aby otworzyć nową kartę z tym katalogiem roboczym (tylko serwer lokalny; upuszczany element musi być folderem).

![Pasek tytułu — przycisk nowej karty i menu profili](images/terminal-usage-creation.jpg)

## Przechwyt myszy i menu kontekstowe

- Kliknięcie prawym przyciskiem w terminalu otwiera jego menu kontekstowe: wklejanie, dzielenie paneli, szybkie polecenia, kolekcje zmiennych środowiskowych, kopiowanie całego tekstu, wiersze konfiguracji MCP i więcej, zależnie od sesji;
- Aplikacje przechwytujące mysz (opencode, pi, vim, tmux, htop, …) dostają zdarzenia myszy zamiast terminala — przytrzymaj `Shift`, by mimo takiej aplikacji wykonywać zwykłe operacje:
  - **Kopiowanie tekstu**: przytrzymaj `Shift` i zaznacz tekst, po czym kliknij gdzie indziej — wciąż trzymając `Shift` — aby odznaczyć; zaznaczenie trafia do schowka;
  - **Menu kontekstowe**: `Shift` + kliknięcie prawym przyciskiem lub `Ctrl` + kliknięcie prawym przyciskiem otwiera menu kontekstowe; kliknięcie prawym przyciskiem przy wciśniętym lewym przycisku też je otwiera (zaznaczenie jest wcześniej kasowane);
  - **Łącza**: `Shift` + klik otwiera łącza jak zwykle.

## Podziały i organizacja przeciąganiem

- **Podział**: kliknij prawym przyciskiem w terminalu → `Split` → `Left` / `Right` / `Top` / `Bottom` dzieli kartę na panele, każdy jako niezależny terminal;

  ![Dzielenie karty — menu Split z prawego przycisku i powstałe panele](images/terminal-usage-split.gif)

- **Scalanie przeciąganiem**: przeciągnij kartę na inny terminal, a pojawią się strefy dokowania (lewo/prawo/góra/dół); upuść, aby scalić kartę z kartą docelową jako nowy panel;

  ![Scalanie przeciąganiem — karta upuszczona na inny terminal staje się nowym panelem](images/terminal-usage-merge.mp4)

- **Odłączenie do okna**: przeciągnij kartę poza pasek kart (lub na inne okno), a cała karta opuści swoje okno jako nowa. Sesje zawsze działają na serwerze — podziały, scalania i przeciągania nigdy nie przerywają uruchomionego polecenia.

## Karty i grupy kart

- **Karta** — jedna karta terminala, być może z kilkoma podzielonymi panelami;
- **Grupa kart** — kontener porządkujący karty. Domyślnie grupy podążają za serwerami: karty serwera lokalnego mieszkają w `Ungrouped`, a każdy udostępniany serwer dostaje grupę `Name (Server)`; obsługiwane są też grupy własne;
- **Grupy i okna** — każde okno pokazuje jedną aktywną grupę przez selektor grup na pasku tytułu (albo `All`); `Move Group to New Window` odrywa całą grupę do własnego okna, poświęcając okno lub monitor jednej maszynie;
- **Grupy i serwery** — grupa pamięta swój serwer, więc przełączanie grup przywraca fokus na przestrzeń roboczą serwera; menu profili podąża za aktywną grupą i pokazuje tylko profile odpowiednich serwerów.

![Selektor grup — lista grup i przeniesienie do nowego okna](images/terminal-usage-groups.jpg)
