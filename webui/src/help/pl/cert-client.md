Domyślnie klient szyfruje każde połączenie, ale nie sprawdza, kto jest po drugiej stronie — ta sama postawa zaufania przy pierwszym użyciu co akceptowanie certyfikatu z podpisem własnym w przeglądarce. **Ścisła weryfikacja** to zmienia: klient sprawdza certyfikat serwera względem głównego CA, które dostarczysz, i odmawia połączenia, dopóki łańcuch, ważność i SAN nie przejdą wszystkich sprawdzeń. To czyni ataki typu man-in-the-middle na połączenie praktycznie niemożliwymi — kosztem jednego ręcznego kroku: przekazania klientowi głównego CA serwera.

## Gdzie to skonfigurować

Ścisłą weryfikację konfiguruje się per serwer w panelu edycji serwera klienta, obok adresu i klucza tajnego:

1. `Certyfikat głównego CA (PEM)` — wielowierszowe pole na publiczny certyfikat głównego CA serwera;
2. `Ścisła weryfikacja CA (łańcuch, ważność i SAN)` — dostępne do zaznaczenia dopiero po wprowadzeniu CA.

![Panel edycji serwera w kliencie desktopowym: wielowierszowe pole Certyfikat głównego CA (PEM) z polem wyboru ścisłej weryfikacji CA poniżej](images/client-ca-fields.jpg)

## Dostarczenie głównego CA do klienta

Dwie drogi, z tego samego źródła (strona `Certyfikaty i TLS` serwera — zobacz stronę Domeny i zarządzanie SAN):

- **Kopiuj i wklej** — naciśnij `Kopiuj główne CA` w interfejsie WWW i wklej PEM w pole `Certyfikat głównego CA (PEM)` klienta;
- **Import ciągu udostępniania** — ciągi udostępniania generowane przez klienta zawierają główne CA razem z adresem i kluczem. Import takiego ciągu automatycznie wstępnie wypełnia pole CA; odbiorca musi tylko zaznaczyć pole wyboru ścisłej weryfikacji.

Główne CA jest takie same dla każdego klienta i nie zmienia się przy odnowieniach certyfikatu — wystarczy ustawić je raz na serwer.

## Co sprawdza tryb ścisły

Gdy pole wyboru jest zaznaczone, każde połączenie z tym serwerem musi przedstawić certyfikat, który:

1. Tworzy łańcuch do skonfigurowanego głównego CA — certyfikat podpisany przez cokolwiek innego zostaje odrzucony;
2. Mieści się w okresie ważności — certyfikaty wygasłe lub jeszcze nieważne zostają odrzucone;
3. Nosi SAN pasujący do adresu, z którym się łączysz — certyfikat wystawiony dla innych nazw zostaje odrzucony.

Jeśli któreś sprawdzenie się nie powiedzie, połączenie zostaje odrzucone z błędem wskazującym, które sprawdzenie nie powiodło się. Bez pola wyboru (lub bez CA) połączenia pozostają szyfrowane, ale te sprawdzenia są pomijane.

## Zweryfikuj odcisk

Zanim zaczniesz ufać głównemu CA, potwierdź, że masz prawdziwe: strona `Certyfikaty i TLS` pokazuje odcisk SHA-256 głównego CA. Porównaj go z PEM-em, który masz zamiar wkleić (lub PEM-em osadzonym w otrzymanym ciągu udostępniania). Jeśli się różnią, nie włączaj ścisłej weryfikacji — zamiast tego uzyskaj CA zaufanym kanałem.

## Częste niepowodzenia

- `Niezgodność SAN` — łączysz się pod nazwą lub adresem IP, którego nie ma w certyfikacie. Dodaj go na serwerze (zobacz stronę Domeny i zarządzanie SAN) i wystaw ponownie;
- `Certyfikat wygasł lub jest jeszcze nieważny` — zegar serwera jest nieprawidłowy albo odnowienie certyfikatu liścia nie powiodło się. Sprawdź status certyfikatu w interfejsie WWW;
- `Nie podpisany przez skonfigurowane CA` — CA w polu należy do innego serwera albo połączenie jest przechwytywane. Porównaj odciski przed kontynuowaniem.
