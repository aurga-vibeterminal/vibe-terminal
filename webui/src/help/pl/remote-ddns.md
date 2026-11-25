Internet domowy i biurowy ma zwykle dynamiczny publiczny IP, zmieniający się bez ostrzeżenia. DDNS trzyma stałą domenę wycelowaną w aktualny IP maszyny — klienci i inne serwery muszą pamiętać tylko nazwę, nigdy adres.

## Jak to działa

![Architektura DDNS — menedżer okresowo wykrywa lokalny IP, dodatek aktualizuje rekord DNS przez API dostawcy, klienci łączą się domeną](diagram/ddns)

Cały potok działa automatycznie w tle serwera:

1. **Menedżer DDNS** odświeża się w stałym cyklu (domyślnie co 5 minut, regulowane zmienną środowiskową `TERMINAL_SERVER_DDNS_POLL_INTERVAL_SECS`); zapis konfiguracji wywołuje natychmiastowe odświeżenie, a nic nie działa, póki usługa jest wyłączona;
2. **Wykrywanie lokalnego IP** czyta najpierw wybrany interfejs sieciowy, wracając do wykrywania po trasach, i filtruje wynik ustawieniem `IP Family` (IPv4 / IPv6 / dual);
3. **Dodatek DDNS** — sandboxowy skrypt JS — dostaje odszyfrowaną konfigurację i aktualny IP i woła API dostawcy DNS przez HTTP. Wbudowany dodatek Cloudflare na przykład znajduje strefę po domenie i przepisuje rekordy A / AAAA na aktualny IP, usuwając przeterminowane rekordy, gdy znika rodzina adresów;
4. **Klienci** rozwiązują domenę do najnowszego IP i łączą się nazwą.

## Konfiguracja

Otwórz sekcję `DDNS` Web UI:

1. Włącz `Enable`. Możesz też zostawić usługę wyłączoną, póki wszystko wypełniasz — wyłączona usługa zapisuje konfigurację, ale nigdy nie działa;
2. wybierz dostawcę pod `Provider`; `Cloudflare DDNS` jest wbudowany;
3. wypełnij pola dostawcy (przykład Cloudflare):
   - `Email` — e-mail konta Cloudflare;
   - `API Token` — token z uprawnieniem Zone → DNS → Edit;
   - `Domain` — domena główna strefy, np. `example.com`;
   - `Subdomain` — nazwa rekordu w strefie, np. `sub` dla `sub.example.com`; zostaw puste dla korzenia strefy;
4. w `Network Interface & IP` wybierz `Interface` i `IP Family`. `Dual Stack` wymaga, by interfejs miał i adres IPv4, i IPv6;
5. kliknij `Save`, potem `Sync Now`, by zweryfikować. `Sync Status` pokazuje ostatnio wypchnięty IP i czas synchronizacji, a przy niepowodzeniu także błąd dodatku.

## Bezpieczeństwo

Dane dostawcy (tokeny API) są szyfrowane przed zapisem (`enc-v2:`) i odszyfrowywane tylko w pamięci, gdy dodatek działa; dodatki wykonują się w sandboxowym runtime JS z wychodzącym HTTP jako jedyną zdolnością.

## Rozszerzanie dostawców

Dostawcy to dodatki: manifest JSON (deklarujący pola konfiguracji, które Web UI zamienia w formularz) plus skrypt JS wystawiający `run()`. Wbudowany `cloudflare-ddns` to implementacja wzorcowa — własnych dostawców można wgrać jako zip w sekcji `Addons`.

## Współpraca z innymi funkcjami

- Gdy domena się rozwiązuje, kod QR sekcji `Server` może przełączyć `Connection Type` na `DDNS Domain`, by klienci skanowali połączenie domenowe;
- inne serwery mogą wstawić `wss://your-domain:port` do swojego `Relay Address`, by gnać łańcuch przekaźnika po domenie (zob. rozdział Relay).
