`Host with HTTP server` publikuje dowolny katalog na serwerze jako usługę plików HTTP jednym kliknięciem: inne urządzenia w tej samej sieci (lub przez przekaźnik) — telefon, inny komputer, stanowisko testowe — mogą przeglądać i pobierać ten katalog z przeglądarki. Idealne do dzielenia się artefaktami buildów i paczkami logów albo wręczania plików urządzeniom bez zainstalowanego klienta.

## Jak używać

Trzy wejścia, wszystkie na katalogu: menu kontekstowe listy plików, menu kontekstowe pozycji Recent i menu kontekstowe okruszków → `Host with HTTP server`.

Kliknięcie:

1. Serwer startuje statyczną usługę HTTP zakorzenioną w tym katalogu, z portem z zakresu Static HTTP;
2. **Serwer lokalny**: adres `http://127.0.0.1:port/` jest kopiowany prosto do schowka, ze statusem „HTTP server: … (skopiowano do schowka)";
3. **Serwer zdalny** (połączony przez przekaźnik): klient automatycznie prosi o mapowanie odwrotnego proxy i kopiuje zewnętrznie osiągalny adres udostępniania — ograniczenia LAN nie są przeszkodą.

Otwórz ten adres w przeglądarce dowolnego urządzenia, a katalog przegląda się i pobiera jak zwykła lista webowa.

## Zarządzanie działającymi usługami

Każda usługa HTTP działa, dopóki nie zostanie zatrzymana:

- Otwórz sekcję `Static HTTP` Web UI; lista `Services` pokazuje każdą działającą usługę i jej katalog dokumentów;
- każda pozycja ma przycisk zatrzymania; zakres portów konfiguruje się na tej samej stronie.

> Usługa jest listą plików tylko do odczytu — bez punktów zapisu; zawartość katalogu widać na żywo w miarę zmian, co pasuje do nieustannie powstających logów i artefaktów.
