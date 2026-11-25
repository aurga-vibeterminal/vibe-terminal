Połącz klienta z tym serwerem za pomocą kodu QR parowania z Web UI — albo wpisując dane ręcznie. Wszystko, czego potrzebuje klient (adres, port, klucz tajny), jest pokazane w sekcji Server.

## Przygotowanie serwera

- Otwórz sekcję `Server` i upewnij się, że ustawiono `Secret Key`; kliknij przycisk kostki, aby go wygenerować. Ponowne wygenerowanie klucza unieważnia każdego sparowanego klienta, który musi sparować się ponownie.
- Rozwiń blok `QR Code` i wybierz `Network Interface` oraz `IP Version` (IPv4 / IPv6), których inne maszyny mają używać, aby dotrzeć do tego serwera.
- Jeśli skonfigurowano domenę DDNS, możesz przełączyć `Connection Type` na `DDNS Domain`, tak by kod QR osadzał domenę zamiast adresu IP.
- `Pairing Preview` pokazuje dokładny adres połączenia (`wss://host:port`), który wybrał klient.
- Kliknięcie `QR code` kopiuje zaszyfrowany ciąg udostępniania do schowka — klienci używają go do automatycznego wypełniania jednym dotknięciem.

## Klient desktopowy

- Kliknij przycisk nazwy serwera na pasku tytułu i wybierz `Servers` w menu, aby otworzyć okno Server Manager.

![Klient desktopowy — menu serwera otwarte kliknięciem przycisku nazwy serwera](images/start-connect-server-menu.jpg)

- Kliknij przycisk `+` obok listy serwerów, aby uruchomić formularz Add Server.
- Jeśli ciąg udostępniania znajduje się w schowku, w prawym górnym rogu formularza pojawia się ikona schowka — kliknij ją, aby odszyfrować ciąg i automatycznie wypełnić `Title`, `Address` i `Secret Key`.
- W przeciwnym razie wypełnij pola ręcznie: nazwę wyświetlaną, adres połączenia (`wss://host:port`, adresy IPv6 w nawiasach) i klucz tajny.
- Kliknij `Save`, a następnie wybierz nowy serwer, aby otworzyć sesję.

![Klient desktopowy — Server Manager z formularzem Add Server](images/start-connect-add-server.jpg)
