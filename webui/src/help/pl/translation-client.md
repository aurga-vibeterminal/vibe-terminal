Po skonfigurowaniu serwera każda powierzchnia klienta oferuje tłumaczenie przez tę samą ikonę globusu. Wyniki są buforowane per element, więc przełączanie tam i z powrotem nigdy nie powtarza żądania.

## Wiadomości w panelu czatu

Każdy nagłówek wiadomości ma ikonę tłumaczenia — obok czasu przy Twoich wiadomościach, na prawo od nazwy nadawcy i czasu przy odpowiedziach modelu oraz na prawo od etykiety `Myśli...` przy blokach rozumowania:

- **Pierwsze kliknięcie** wysyła surowy Markdown wiadomości na serwer i zamienia treść wiadomości na tłumaczenie. Ikona świeci, póki tłumaczenie jest widoczne;
- **Kolejne kliknięcie** przywraca oryginalny tekst;
- Tłumaczenie trzyma się wiadomości, póki rozmowa pozostaje otwarta, łącznie z blokami `Myśli...` po rozwinięciu.

Przetłumaczone wiadomości renderują się jak zwykły Markdown, więc listy, tabele i bloki kodu zachowują kształt.

![Wiadomość czatu z ikoną tłumaczenia w nagłówku i przetłumaczoną treścią poniżej](images/translation-chat-message.jpg)

## Szkic komunikatu commitu

W polu commitu panelu Git przycisk tłumaczenia pojawia się obok przycisku AI, gdy tylko szkic przekroczy jeden wiersz:

- **Kliknij**, by przetłumaczyć cały szkic — przetłumaczony tekst zastępuje szkic w polu jako podgląd tylko do odczytu;
- **Kliknij ponownie**, by wrócić do pierwotnego brzmienia (sam szkic nigdy nie został zmieniony);
- Pisanie jest zablokowane podczas podglądu; najpierw przełącz z powrotem, potem edytuj dalej.

![Pole commitu z przyciskiem tłumaczenia i przetłumaczonym podglądem](images/translation-commit-draft.jpg)

## Okienko szczegółów commitu

Najedź na commit w historii Gita, by otworzyć okienko szczegółów. Obok przycisku kopiowania hasha siedzi ikona tłumaczenia: kliknij, by przetłumaczyć komunikat commitu w okienku; kliknij ponownie, by zobaczyć oryginał. Przetłumaczony komunikat renderuje się jak oryginał — wiersz tematu, treść Markdown i bloki kodu włącznie.

![Okienko szczegółów commitu — przyciski kopiowania i tłumaczenia w wierszu hasha](images/translation-git-tip.jpg)

## Zaznaczenia w edytorze kodu

Zaznacz tekst w edytorze kodu — zwykłe i blokowe (prostokątne) zaznaczenia działają — kliknij prawym przyciskiem i wybierz `Przetłumacz` (pierwsza pozycja menu):

1. Wypełniona plakietka globusu pojawia się nad numerem wiersza pierwszego zaznaczonego wiersza;
2. Karta tłumaczenia otwiera się od razu z symbolem zastępczym *Translating…* i wypełnia, gdy serwer odpowie;
3. Karta jest przyklejona: zostaje na miejscu podczas przewijania, a odrzuca ją tylko przycisk zamknięcia, `Esc` albo kliknięcie na zewnątrz. Najechanie na plakietkę przywołuje ją z powrotem.

Każde przetłumaczone zaznaczenie ma własną plakietkę, a ponowne przetłumaczenie tych samych wierszy po prostu pokazuje ponów buforowaną kartę. Plakietki znikają po otwarciu innego pliku.

![Zaznaczenie w edytorze kodu z Przetłumacz w menu kontekstowym](images/translation-editor-menu.jpg)

## Zaznaczenia w widoku diffu

Widok diffu działa tak samo: zaznacz wiersze w treści diffu, prawy przycisk, `Przetłumacz`. Plakietka zastępuje nowy numer wiersza pierwszego zaznaczonego wiersza, a karta wyniku zakotwicza się pod nim. Zaznaczenia w trybie edycji też się tłumaczą; przełączenie na inny plik lub commit czyści plakietki.

![Diff z przetłumaczonym zaznaczeniem — plakietka w marginesie i karta tłumaczenia](images/translation-diff-card.jpg)

## Co jest tłumaczone

Serwer mówi modelowi, skąd pochodzi tekst:

- **Proza** (wiadomości czatu, komunikaty commitów) — formatowanie Markdown jest zachowane, tłumaczony jest tylko język naturalny;
- **Zaznaczenia z plików** (edytor, diff) — nazwa pliku podróżuje z żądaniem, więc model zna znaczniki komentarzy tego pliku (`//`, `#`, `--`, …). Znaczniki komentarzy są usuwane, a tekst komentarza tłumaczony jak proza; Markdown wewnątrz komentarza (nagłówki, listy, bloki kodu) przeżywa. Jeśli zaznaczenie to zwykły kod, tłumaczone są tylko komentarze i literały łańcuchowe — identyfikatory i struktura pozostają nietknięte.
