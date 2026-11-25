Serwer chroni wszystkie udostępniane punkty końcowe TLS — interfejs WWW, połączenia klientów i przekierowane porty — certyfikatem wystawionym przez własne **prywatne główne CA**. Strona `Certyfikaty i TLS` w interfejsie WWW to miejsce, w którym zarządzasz tym CA, określasz, dla jakich nazw certyfikat jest ważny, i przekazujesz CA klientom, które mają ściśle weryfikować serwer.

## Jak to działa

- Serwer generuje prywatne główne CA dokładnie raz i przechowuje je przez cały okres życia instalacji. Wszystko jest przez nie podpisywane;
- Główne CA podpisuje dla serwera **certyfikat liścia** o ważności 90 dni, obejmujący nazwy hostów i adresy IP, których klienci faktycznie używają, aby się z nim łączyć;
- Gdy zmienią się nazwy hostów, interfejsy sieciowe lub wpisy SAN maszyny, certyfikat liścia jest wystawiany ponownie automatycznie — główne CA nigdy się nie zmienia, więc klienci, którzy mu już ufają, nadal ufają serwerowi;
- Nowy certyfikat liścia jest podmieniany przez współdzielony resolwer certyfikatów i wczytywany na gorąco: istniejące połączenia nie są nigdy rozłączane, a nowe uzgadniania od razu używają najnowszego certyfikatu. Jeśli odnowienie się nie powiedzie, poprzedni certyfikat pozostaje aktywny.

![Strona Certyfikaty i TLS: u góry karta Prywatne główne CA, w środku zarządzanie SAN, u dołu szczegóły certyfikatu liścia serwera](images/certificates-page.jpg)

## Co znajdziesz na stronie

1. `Prywatne główne CA` — pobierz lub skopiuj publiczny certyfikat CA, aby rozdystrybuować go do klientów (i ich przeglądarek). Pobrany plik nigdy nie zawiera klucza prywatnego;
2. `Domeny i adresy IP (SAN)` — lista nazw, dla których certyfikat liścia jest ważny. Opisana na następnej stronie, Domeny i zarządzanie SAN;
3. `Certyfikat liścia serwera` — aktywny certyfikat: status, podmiot, odcisk SHA-256, okres ważności i wpisy SAN.

## Niestandardowe certyfikaty TLS

Jeśli dla serwera skonfigurowano niestandardowy certyfikat TLS, to on pozostaje nadrzędny: certyfikat zarządzany na tej stronie nadal jest generowany i utrzymywany w aktualności, ale nie zastępuje skonfigurowanej niestandardowej tożsamości. Dopóki niestandardowa tożsamość jest aktywna, strona pokazuje powiadomienie.

## Dlaczego ufać głównemu CA

Klienci, którzy potrzebują wyłącznie szyfrowania, działają bez dodatkowej konfiguracji. Klienci, którzy chcą *weryfikować* serwer — tryb ścisły, opisany na ostatniej stronie, Ścisła weryfikacja na klientach — potrzebują publicznego certyfikatu głównego CA, a ten musi dotrzeć kanałem, któremu ufasz:

> Pobieraj główne CA wyłącznie z lokalnego interfejsu WWW tego serwera lub przez zaufany tunel i zweryfikuj odcisk SHA-256 (pokazany na tej stronie), zanim włączysz ścisłą weryfikację gdziekolwiek.

To samo CA podpisuje też certyfikaty używane przez przekierowanie portów (zobacz stronę Przekierowanie portów). Gdy urządzenie zacznie ufać głównemu CA, przekierowane adresy URL otworzą się bez ostrzeżenia przeglądarki *połączenie nie jest prywatne*.
