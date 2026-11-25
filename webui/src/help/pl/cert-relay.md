Domyślnie serwer udostępniany szyfruje swoje połączenie nadrzędne z przekaźnikiem, ale nie sprawdza, kto jest po drugiej stronie — ta sama postawa zaufania przy pierwszym użyciu co akceptowanie certyfikatu z podpisem własnym w przeglądarce. **Rygorystyczne TLS przekaźnika** to zmienia: serwer udostępniany sprawdza certyfikat przekaźnika względem głównego CA, które dostarczysz, i odmawia połączenia, dopóki łańcuch, ważność i nazwa hosta nie przejdą wszystkich sprawdzeń. To czyni ataki typu man-in-the-middle na łącze przekaźnika praktycznie niemożliwymi — kosztem jednego ręcznego kroku: przekazania serwerowi udostępnianemu głównego CA przekaźnika. Stosuje tę samą ideę co poprzednia strona, Ścisła weryfikacja na klientach, do nadrzędnego łącza przekaźnika serwera udostępnianego.

## Gdzie to skonfigurować

Ścisłą weryfikację konfiguruje się na serwerze udostępnianym — maszynie, której `Relay Address` wskazuje na przekaźnik. W jego interfejsie WWW otwórz sekcję `Server`: pod polem `Relay Secret Key` znajduje się wiersz `Rygorystyczne TLS przekaźnika` pokazujący bieżący stan `On` / `Off`. Rozwiń go:

1. `Główny CA nadrzędnego przekaźnika` — wielowierszowe pole na publiczny certyfikat głównego CA przekaźnika;
2. Przełącznik `Off` / `On` — `On` można wybrać dopiero, gdy PEM da się odczytać jako prawidłowy certyfikat.

![Web UI — rozwinięty blok Rygorystyczne TLS przekaźnika w sekcji Server: pole Główny CA nadrzędnego przekaźnika z przełącznikiem Off/On i przyciskiem Apply](images/cert-relay-strict-tls.jpg)

Ustawienia przekaźnika nigdy nie zapisują się w trakcie edycji: przejrzyj wartości i kliknij `Apply`. Zapis natychmiast łączy ponownie z nadrzędnym przekaźnikiem przy nowych ustawieniach.

## Dostarczenie głównego CA do serwera udostępnianego

Dwie drogi, z tego samego źródła (interfejs WWW samego przekaźnika):

- **Wklejenie ciągu udostępniania** — kliknij `QR code` w interfejsie WWW serwera przekaźnika. Ciąg udostępniania zawiera główne CA przekaźnika razem z adresem i kluczem, więc wklejenie go w pole `Relay Address` serwera udostępnianego automatycznie wypełnia także pole CA (zobacz Przewodnik konfiguracji w sekcji Przekaźnik (Relay)). Tryb ścisły pozostaje wyłączony, dopóki sam nie sprawdzisz CA i go nie włączysz;
- **Kopiuj i wklej** — naciśnij `Kopiuj główne CA` na stronie `Certyfikaty i TLS` przekaźnika i wklej PEM w pole `Główny CA nadrzędnego przekaźnika`.

Główne CA jest takie same dla każdego serwera udostępnianego i nie zmienia się przy odnowieniach certyfikatu — wystarczy ustawić je raz na przekaźnik.

Zanim zaczniesz ufać głównemu CA, potwierdź, że masz prawdziwe: strona `Certyfikaty i TLS` przekaźnika pokazuje odcisk SHA-256 głównego CA. Porównaj go z PEM-em, który masz zamiar wkleić. Jeśli się różnią, nie włączaj ścisłej weryfikacji — zamiast tego uzyskaj CA zaufanym kanałem.

## Co sprawdza tryb ścisły

Gdy przełącznik jest na `On`, nadrzędne połączenie `RelayPrimary` musi przedstawić certyfikat, który:

1. Tworzy łańcuch do skonfigurowanego głównego CA — certyfikat podpisany przez cokolwiek innego, w tym publiczne główne systemowe, zostaje odrzucony;
2. Mieści się w okresie ważności — certyfikaty wygasłe lub jeszcze nieważne zostają odrzucone;
3. Nosi SAN pasujący do hosta adresu `Relay Address`, z którym się łączysz — certyfikat wystawiony dla innych nazw zostaje odrzucony.

Jedno ograniczenie: weryfikowane jest tylko nadrzędne połączenie z przekaźnikiem. Połączenia zwrotne Terminal i żądań zachowują dotychczasową politykę certyfikatów.

Jeśli któreś sprawdzenie się nie powiedzie, połączenie zostaje odrzucone i ponawiane w tle z rosnącymi odstępami; serwer udostępniany pozostaje nieosiągalny przez przekaźnik, aż sprawdzenie przejdzie albo wyłączysz tryb ścisły.

## Częste niepowodzenia

- `Ścisła weryfikacja TLS wymaga PEM głównego CA` — przełącznik jest włączony, ale pole CA jest puste. Najpierw wklej PEM;
- `Niezgodność SAN` — łączysz się z przekaźnikiem pod nazwą lub adresem IP, którego nie ma w jego certyfikacie. Dodaj ją na stronie `Certyfikaty i TLS` przekaźnika (zobacz stronę Domeny i zarządzanie SAN) i wystaw ponownie;
- `Certyfikat wygasł lub jest jeszcze nieważny` — zegar przekaźnika jest nieprawidłowy albo odnowienie certyfikatu liścia nie powiodło się. Sprawdź status certyfikatu na przekaźniku;
- `Nie podpisany przez skonfigurowane CA` — CA w polu należy do innego przekaźnika albo połączenie jest przechwytywane. Porównaj odciski przed kontynuowaniem.
