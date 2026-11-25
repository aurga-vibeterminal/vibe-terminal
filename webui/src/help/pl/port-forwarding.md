Przekierowanie portów publikuje usługę HTTP działającą na maszynie serwera — serwer deweloperski, panel urządzenia, cokolwiek nasłuchującego pod zwykłym adresem `http://localhost:port` — na własnym porcie TLS, więc każde urządzenie, które dociera do serwera, otwiera ją w przeglądarce jako `https://<adres-serwera>:<port>/`.

## Jak to działa

- Przekierowanie mapuje **port docelowy** — usługę HTTP na tej maszynie — na **port przekierowany**, przydzielany ze skonfigurowanego zakresu portów (domyślnie `62000`–`62999`);
- port przekierowany mówi TLS: cały ruch jest szyfrowany **samopodpisanym certyfikatem** serwera — nic nie wędruje w postaci jawnej, nawet przez niezaufaną sieć;
- każde urządzenie, które dociera do serwera — w sieci LAN albo z internetu, gdy zakres portów jest przekierowany na routerze — otwiera `https://<adres-serwera>:<portPrzekierowany>/` i ląduje na lokalnej usłudze.

Przekierowania powstają na żądanie: udostępnienie plików `Host with HTTP server` na zdalnym (przekaźnikowym) serwerze automatycznie prosi o jedno i podaje gotowy do otwarcia adres URL. Żadna ręczna konfiguracja nie jest potrzebna.

## Zarządzanie przekierowaniami

![Web UI — sekcja Przekierowanie portów: konfiguracja zakresu portów i lista działających przekierowań](images/port-forwarding-webui.jpg)

1. Otwórz sekcję `Port Forwarding` Web UI serwera.
2. `Port Range Configuration` ogranicza pulę, z której przydzielane są przekierowane porty; zmiany zapisują się automatycznie. Zawęź ją, gdy zapora lub router przekierowuje tylko kilka portów.
3. Lista na żywo pokazuje każde przekierowanie jako `:portPrzekierowany → :portDocelowy` wraz z liczbą aktywnych połączeń.
4. Przycisk zatrzymania przekierowania natychmiast zdejmuje nasłuch i rozłącza jego aktywne połączenia; następne udostępnienie przydzieli świeży port. `Refresh` ponownie odczytuje listę.

## Ostrzeżenie o certyfikacie

Przekierowania są szyfrowane własnym samopodpisanym certyfikatem serwera — tym samym, który chroni Web UI i połączenia klientów. Przeglądarki nie mogą zweryfikować go względem urzędu certyfikacji, więc pierwsza wizyta pod przekierowanym adresem URL pokazuje ostrzeżenie o prywatności (*Połączenie nie jest prywatne*). To oczekiwane i bezpieczne do przekliknięcia: samo szyfrowanie nie jest złamane, jedynie pochodzenia certyfikatu nie da się zweryfikować maszynowo. Aby wyciszyć ostrzeżenie na stałe, zaimportuj certyfikat serwera do magazynu zaufanych certyfikatów urządzenia.

![Przeglądarka — ostrzeżenie o niezaufanym samopodpisanym certyfikacie; stronę otwiera się przez Zaawansowane → Przejdź dalej](images/port-forwarding-cert-warning.jpg)

## Chrome na macOS pokazuje `ERR_ADDRESS_UNREACHABLE`

Od macOS 15 (Sequoia) aplikacje firm trzecich potrzebują wyraźnego pozwolenia na docieranie do urządzeń w sieci lokalnej — każdego adresu `192.168.x.x`, `10.x.x.x` i podobnego. Safari jest zwolnione jako aplikacja systemowa, więc przekierowany URL może otwierać się w Safari bez problemu, podczas gdy Chrome zgłasza `ERR_ADDRESS_UNREACHABLE`, zanim TLS w ogóle się zacznie. To nie jest problem certyfikatu ani problem samego przekierowania.

Aby przyznać Chrome dostęp:

1. Otwórz `Ustawienia systemowe` → `Prywatność i ochrona` → `Sieć lokalna`.
2. Znajdź `Google Chrome` na liście i **włącz** go.
3. Całkowicie zamknij Chrome (`Cmd+Q`, nie tylko zamknięcie okien) i uruchom ponownie.

Jeśli przełącznik jest już włączony, wyłącz go i włącz ponownie — potrafi się zaciąć po aktualizacji Chrome lub macOS. Jeśli błąd nie ustępuje, sprawdź, czy w Chrome nie działa rozszerzenie VPN lub proxy, które nie routuje prywatnych zakresów adresów.

## Kontrola dostępu

To, co chroni przekierowany port, zależy od tego, co za nim stoi:

- **Wbudowane udostępnienia wymagają tokenu.** Linki udostępniania generowane przez klienta — udostępnienie `Host with HTTP server` (`?share_token=…`) i strona Settings (`?ticket=…`) — zawierają token dostępu. Połączenie bez ważnego tokenu i bez ważnej sesji jest zrywane, zanim cokolwiek zostanie wysłane. Przy pierwszej wizycie przeglądarka wymienia token na bezpieczne ciasteczko sesyjne działające tylko dla tej usługi, a token znika z adresu URL — nie zostaje ani w pasku adresu, ani w historii.

- **Tokeny są jednorazowe i krótkotrwałe.** Ticket Settings działa dokładnie raz i wygasa po minucie; gdy udostępnienie zostanie zatrzymane lub uruchomione ponownie, wcześniejsze linki i sesje przestają działać. Traktuj pełny link jak dane uwierzytelniające — kto go ma, ten ma dostęp do usługi — i udostępniaj go wyłącznie osobom, którym ufasz.

- **Własne usługi chroni ich własne uwierzytelnianie.** Przekierowanie do dowolnej innej lokalnej usługi szyfruje ruch samodzielnie podpisanym certyfikatem serwera, ale nie dodaje uwierzytelniania: wszystko, co serwuje usługa — strony i API — jest dostępne dla każdego, kto dotrze do portu. Zabezpiecz uwierzytelnianiem wrażliwe usługi, zanim je eksponujesz.
