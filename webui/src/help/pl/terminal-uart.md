UART zamienia konsolę szeregową urządzenia (port debugowania, płytkę wbudowaną, SBC) w kartę terminala, którą można oglądać i obsługiwać jak każdy inny terminal.

## Konfiguracja portu

W sekcji `UART` Web UI → `Add UART Port`:

- `Name` — widoczny jako tytuł terminala (np. „Debug Console");
- `Serial Port` — wybierz z wykrytych portów (liczba jest pokazana na górze strony);
- `Baud Rate` / `Data Bits` / `Parity` / `Stop Bits` / `Flow Control` — dopasuj do firmware'u urządzenia;
- `Terminal Type` i `Encoding` — kodowanie domyślnie Auto (UTF-8) i może podążać za firmware'em;
- `Show Advanced Options` odsłania dalsze parametry szeregowe.

Zapisane porty pojawiają się na liście `Configured UART Ports` i można je w każdej chwili edytować lub usunąć.

![Strona ustawień UART — formularz portu](images/terminal-uart-config.jpg)

## Użycie w kliencie

Skonfigurowane porty otwierają się jako karty terminala jak zwykłe sesje serwera:

- Dane szeregowe są wyświetlane po nadejściu; wejście z klawiatury idzie prosto do portu;
- Menu kontekstowe oferuje `Clear` (wyczyszczenie ekranu) i podmenu `Encoding` (zmiana kodowania na żywo; bieżąca jest zaznaczona);
- Sesje UART nie mają funkcji powłoki (brak Szybkich poleceń, Apps czy pozycji menu Envs).

Typowe zastosowania: konsole routerów/przełączników, logi płytek wbudowanych, nadzór programatorów.
