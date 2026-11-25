Podłączenie maszyny do przekaźnika zajmuje dwa pola: adres przekaźnika i jego klucz. Co to za funkcja i jak elementy do siebie pasują — zob. poprzednią stronę, Przegląd.

## Konfiguracja serwera przekaźnika

1. Otwórz sekcję `Server` na maszynie, która ma być przekaźnikiem.
2. Upewnij się, że ustawiono `Secret Key` — przycisk kostki go wygeneruje. Ten sam klucz paruje klientów i udostępniane serwery.
3. Rozwiń blok `QR Code` i wybierz `Network Interface` / `IP Version`, których klienci i udostępniane serwery mają użyć, by do niego dotrzeć.

## Podłączenie serwera udostępnianego

Dane połączenia przekaźnika podróżują w schowku — bez przepisywania:

1. W Web UI serwera przekaźnika kliknij `QR code`. Kopiuje to zaszyfrowany ciąg udostępniania (adres, port, klucz tajny) do schowka.
2. Otwórz Web UI na udostępnianej maszynie, sekcję `Server`, i kliknij w pole `Relay Address`.
3. Przeglądarka pokazuje okno z pytaniem, czy zezwolić tej witrynie na dostęp do schowka — kliknij `Allow`:

![Przeglądarka — okno zezwolenia na dostęp do schowka przy skupieniu pola](images/relay-allow-paste.jpg)

4. Po zezwoleniu schowek jest sprawdzany: gdy zawiera ciąg udostępniania, obok pola pojawia się przycisk wklejania — kliknij go, by odszyfrować i automatycznie wypełnić `Relay Address` i `Relay Secret Key`:

![Web UI — adres przekaźnika i klucz tajny wypełnione automatycznie z wklejonego ciągu](images/relay-paste-share.jpg)

5. Przejrzyj wartości i kliknij `Apply` — zapis ponownie łączy z nadrzędnym przekaźnikiem.

![Web UI — wypełnione pola przekaźnika z przyciskiem Apply](images/relay-apply-share.jpg)

> Czytanie schowka wymaga bezpiecznego kontekstu (HTTPS albo localhost). Na czystym HTTP przycisk wklejania pozostaje ukryty; wpisz adres i klucz przekaźnika ręcznie.

Domyślnie połączenie nadrzędne jest szyfrowane, ale nie weryfikowane. Aby przypiąć także certyfikat przekaźnika, zobacz Ścisła weryfikacja na przekaźniku w sekcji Certyfikaty i TLS.

## Klient desktopowy

### Karty serwerów udostępnianych

Karty terminali działających na udostępnianym serwerze niosą dwa znaczniki: kolorową smugę na lewej krawędzi karty — jeden kolor na serwer — i ikonę zdalnego OS (Windows / Apple / Linux) obok kropki statusu. Lokalne karty samego serwera przekaźnika nie mają znaczników.

![Klient desktopowy — znaczniki kart terminali serwerów udostępnianych](images/relay-tab-indicator.jpg)

### Profile terminali per serwer

Menu profili (pasek tytułu) grupuje profile terminali według serwera: najpierw własne profile podłączonego serwera przekaźnika, potem po jednej rozwijanej grupie na serwer udostępniany, nazwanej od maszyny. Rozwiń grupę i wybierz profil, by otworzyć terminal na tym serwerze. Menu podąża za grupą kart wybraną na pasku tytułu.

![Klient desktopowy — menu profili z grupami per serwer](images/relay-profile-menu.jpg)

### Grupowanie i osobne okna

Selektor grup na pasku tytułu wypisuje grupy kart wszystkich serwerów; domyślna grupa serwera udostępnianego pokazuje się jako `Ungrouped (Nazwa serwera)`. Podmenu każdej grupy wypisuje jej karty do szybkiego przełączania i oferuje `Move Group to New Window`, które odrywa całą grupę do własnego okna — poręczne, by poświęcić okno albo monitor jednej maszynie.

![Klient desktopowy — menu selektora grup z akcją przeniesienia do nowego okna](images/relay-group-selector.jpg)
