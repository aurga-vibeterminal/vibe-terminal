Tłumaczenie zamienia tekst napotykany w kliencie na wybrany przez Ciebie język: odpowiedzi czatu AI, komunikaty commitów, komentarze w kodzie albo dowolny zaznaczenie w edytorze kodu lub diffie. Pracę wykonuje model tłumaczeniowy skonfigurowany na serwerze — klient tylko wysyła tekst, renderuje wynik i buforuje go, byś mógł przełączać się między oryginałem a tłumaczeniem.

## Gdzie tłumaczenie jest dostępne

- **Panel Copilot** — każdy nagłówek wiadomości (Twoje wiadomości, odpowiedzi modelu i bloki `Myśli...`) ma ikonę tłumaczenia. Jedno kliknięcie zastępuje treść wiadomości tłumaczeniem; kolejne przywraca oryginał;
- **Szkic komunikatu commitu** — pole commitu w panelu Git pokazuje przycisk tłumaczenia, gdy tylko szkic rozrośnie się na kilka wierszy. Tłumaczy cały szkic w miejscu, a kolejne kliknięcie przywraca pierwotne brzmienie;
- **Wyskakujące okienko szczegółów commitu** — najechanie na commit w historii Gita otwiera okienko szczegółów; ikona tłumaczenia obok hasha przełącza komunikat commitu na Twój język;
- **Edytor kodu i widok diffu** — zaznacz cokolwiek (zaznaczenia blokowe też się liczą), kliknij prawym przyciskiem i wybierz `Przetłumacz`. Mała plakietka globusu oznacza przetłumaczone wiersze w marginesie, a obok zaznaczenia otwiera się karta z tłumaczeniem.

## Jak to działa

1. Klient wysyła tekst na serwer — wraz z nazwą pliku, gdy tekst zaznaczono w edytorze lub diffie, by model wiedział, jakiej składni komentarzy używa plik;
2. LLM tłumaczeniowy serwera tłumaczy go na język docelowy skonfigurowany w interfejsie WWW. Formatowanie Markdown (nagłówki, listy, tabele, bloki kodu) jest zachowane; w zaznaczeniach z plików znaczniki komentarzy takie jak `//` czy `#` są rozpoznawane jako dekoracja i usuwane, a kod i identyfikatory pozostają nietknięte;
3. Klient renderuje wynik jako Markdown tam, skąd przyszło żądanie — w miejscu dla wiadomości czatu albo w przewijanej karcie dla zaznaczeń edytora i diffów.

## Czego potrzebujesz

- Serwera z włączoną funkcją tłumaczenia — jednorazowa konfiguracja opisana jest w *Konfiguracji serwera* (sekcja `LLM` interfejsu WWW);
- Klienta desktopowego połączonego z tym serwerem. Po stronie klienta niczego się nie konfiguruje: wszystkie powierzchnie automatycznie przejmują te same ustawienia serwera.

Jeśli tłumaczenie nie jest skonfigurowane, klient pokazuje błąd serwera (np. w karcie tłumaczenia) zamiast wyniku.
