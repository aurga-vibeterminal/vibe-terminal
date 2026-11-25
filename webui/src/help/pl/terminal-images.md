Zrzuty ekranu i terminal spotykają się w jednym przepływie: jeden globalny skrót chwyta dowolny obszar pulpitu, oznaczasz go adnotacjami i kopiujesz albo zapisujesz — potem wklejasz obraz ze schowka prosto do terminala, gdzie ląduje jako ścieżka pliku, którą działający program (też agent kodu) może otworzyć i użyć. Obie połowy żyją w kliencie desktopowym.

## Robienie zrzutu ekranu

- Naciśnij `Shift`+`Alt`+`A` gdziekolwiek: skrót jest globalny, więc reaguje niezależnie od tego, która aplikacja ma fokus, dopóki klient desktopowy działa (na macOS naciśnij `Shift`+`Command`+`A`);
- Każdy ekran przygasa pod nakładką przechwytywania. Przeciągnij, by narysować zaznaczenie — może rozpiętość na kilka ekranów — potem przeciągaj wewnątrz niego, by je przenieść, albo ciągnij jego uchwyty, by zmienić rozmiar; plakietka nad zaznaczeniem pokazuje rozmiar w pikselach;
- `Esc` w każdej chwili przerywa.

![Nakładka przechwytywania — zaznaczenie z plakietką rozmiaru i paskiem adnotacji](images/terminal-images-capture.jpg)

## Adnotowanie

Pasek pod zaznaczeniem niesie po lewej narzędzia adnotacji (prostokąt i elipsa), w środku sześć kolorów, a po prawej trzy przyciski akcji — anuluj (`✕`), zapisz (strzałka pobierania) i kopiuj (`✓`).

- Wybierz narzędzie i kolor, potem przeciągaj wewnątrz zaznaczenia, by rysować; kreski są przycinane do zaznaczenia;
- `Ctrl`+`Z` cofa ostatnią adnotację, `Ctrl`+`R` ją przywraca;
- Przeciągnięcie nowego obszaru poza bieżącym zaznaczeniem wyrzuca je — wraz z adnotacjami — i zaczyna nowe.

## Kopiowanie lub zapisywanie

- `Kopiuj` (`✓`) kładzie zanotowany zrzut do schowka — gotowy do wklejenia w terminal (poniżej), czat lub każdą inną aplikację;
- `Zapisz` (strzałka pobierania) zapisuje PNG przez okno zapisu; nazwa pliku domyślnie brzmi `vterm-screenshot-<timestamp>.png`.

## Wklejanie obrazu do terminala

- Skopiuj obraz do schowka — z narzędzia zrzutu powyżej, z przeglądarki albo z własnego przechwytywania systemu;
- Kliknij terminal prawym przyciskiem i wybierz `Paste Image`; wpis pojawia się tylko wtedy, gdy schowek faktycznie zawiera obraz;
- Obraz jest wysyłany na serwer sesji i przechowywany tam jako plik tymczasowy; jego ujęta w cudzysłów ścieżka zostaje potem wpisana w kursor terminala dokładnie tak, jakbyś ją wpisał — owinięta w znaczniki wklejania w nawiasach, gdy działająca aplikacja je wspiera, żeby powłoki, edytory i agenci dostali ją czysto;
- Dla programu to zwykły tekst: agent kodu taki jak Claude Code czy opencode otworzy obraz po ścieżce, a zwykłe polecenia powłoki mogą użyć jej jako argumentu.

![Prawy przycisk terminala — wpis Paste Image](images/terminal-images-paste.jpg)

## Uwagi

- Wstawiona ścieżka żyje na serwerze sesji: na zdalnym lub WSL-serwerze plik tymczasowy powstaje na tamtej maszynie, dokładnie tam, gdzie programy działające w terminalu mogą go sięgnąć;
- W połączeniu z `Dodawanie kontekstu dla agentów` to najszybszy sposób, by pokazać agentowi coś na ekranie — zrzuć, wklej, a agent trzyma obraz jako plik;
- Pod Linuksem przechwytywanie działa pod X11; natywne sesje Wayland nie wspierają ani globalnego skrótu, ani interaktywnej nakładki, a jeśli inna aplikacja X11 zajęła już `Shift`+`Alt`+`A`, skrót nie da się zarejestrować.
