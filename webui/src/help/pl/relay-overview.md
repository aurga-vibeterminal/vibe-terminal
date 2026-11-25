Przekaźnik rozwiązuje jeden problem: większość maszyn siedzi za NAT-em lub dynamicznymi IP, więc klienci nie mogą dzwonić do każdej wprost. Niech jedna publicznie osiągalna maszyna pełni punkt wejścia przekaźnika; każda inna dzwoni do niej na zewnątrz. Klient utrzymuje wtedy jedno połączenie z przekaźnikiem i dociera przezeń do terminali każdego udostępnianego serwera.

## Architektura

![Architektura przekaźnika — każdy klient trzyma jedno połączenie z przekaźnikiem; udostępniane serwery dzwonią do przekaźnika w górę](diagram/relay)

- **Serwer przekaźnika** — publiczny punkt wejścia, z którym łączą się klienci i udostępniane serwery. Zostaw jego `Relay Address` puste.
- **Serwer udostępniany** — maszyna, której `Relay Address` wskazuje na przekaźnik. Dzwoni w górę i udostępnia swoje terminale przez niego. Kolorowa smuga na każdym udostępnianym węźle diagramu odzwierciedla smugę, którą klient desktopowy maluje na kartach tego serwera — jeden kolor na serwer.
- Połączenia zawsze powstają od środka: udostępniane serwery dzwonią do przekaźnika, więc nie potrzebują przekierowania portów ani publicznego IP.

## Co widzi klient

Po połączeniu z przekaźnikiem każdy udostępniany serwer zachowuje się jak maszyna lokalna:

- Karty terminali niosą kolorową smugę i ikonę zdalnego OS, identyfikując maszynę jednym rzutem oka (zob. Przewodnik konfiguracji);
- menu profili grupuje terminale per serwer — otwórz terminal na dowolnej maszynie wprost;
- grupy kart można odrywać do własnych okien, poświęcając okno jednej maszynie.

## Model bezpieczeństwa

Klienci i udostępniane serwery parują się tym samym kluczem tajnym; połączenia są tunelowane przez przekaźnik. Klucz podróżuje w kodzie QR parowania / ciągu udostępniania, nigdy jawnym tekstem.

Instrukcje krok po kroku są na następnej stronie, Przewodnik konfiguracji.
