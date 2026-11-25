Web UI to strona konfiguracyjna samego serwera: każda funkcja serwera jest pogrupowana w sekcje wypisane na lewym pasku bocznym. To nie terminal — codzienna praca dzieje się w kliencie desktopowym lub `vtermc` (zob. Przegląd). Ta strona wyjaśnia układ i typowe zachowanie; szczegółowy opis ustawień każdej sekcji znajduje się w rozdziałach o funkcjach.

## Układ

- **Pasek boczny** — lista sekcji (poniżej), ikona pomocy obok nazwy serwera i wersja aplikacji na dole; na wąskich ekranach wysuwa się z lewej, a jego widoczność przełącza przycisk hamburgera w nagłówku;
- **Nagłówek** — tytuł bieżącej sekcji oraz, w prawym górnym rogu, selektor języka i przełącznik motywu jasny/ciemny (na wąskich ekranach ikona pomocy siedzi tam również);
- **Zawartość** — formularz wybranej sekcji, wyśrodkowany i przewijany.

## Sekcje

W kolejności paska bocznego:

- `Serwer` — ustawienia portu, certyfikatu TLS i uwierzytelniania: tu mieszkają `Secret Key`, `QR Code` do parowania, adres przekaźnika oraz przełącznik `Auto-start on Login` (zob. Łączenie klienta i Przewodnik konfiguracji). Na Linuksie wskazówka autostartu wyjaśnia też automatyczne logowanie menedżera wyświetlania, by dotrzeć do pulpitu przed lokalnym zalogowaniem (zob. Konfiguracja serwera w rozdziale Pulpit zdalny i symulatory);
- `Terminal` — analiza terminala: wykrywanie prompta, automatyczne potwierdzanie i klasyfikacja LLM (zob. Automatyczne potwierdzanie);
- `UART` — profile terminali szeregowych oparte na lokalnych portach UART (zob. Port szeregowy UART);
- `Dodatki` — zainstalowane dodatki: włączaj, edytuj lub przesyłaj je (dostawcy LLM to dodatki — zob. LLM);
- `LLM` — przypisuje dostawcę LLM do każdej funkcji AI (zob. LLM);
- `Głos` — dostawca rozpoznawania mowy, model i ustawienia transkrypcji (zob. Konfiguracja serwera w rozdziale Wprowadzanie głosowe);
- `Środowisko` — kolekcje zmiennych środowiskowych w drzewie grup (zob. Zmienne środowiskowe);
- `MCP` — serwer MCP terminala udostępniony agentom kodu (zob. Terminal MCP);
- `DDNS` — dostawca DDNS i ustawienia synchronizacji (zob. DDNS);
- `Przekierowanie portów` — pula przekierowanych portów i działające przekierowania TLS (zob. Przekierowanie portów);
- `Static HTTP` — katalogi publikowane przez HTTP i aktywne udostępnienia plików (zob. Publikacja przez HTTP);
- `Pulpit zdalny` — ustawienia przechwytywania, wejścia i transmisji pulpitu zdalnego (zob. Konfiguracja serwera w rozdziale Pulpit zdalny i symulatory);
- `Statystyki LLM` — zużycie tokenów, liczby żądań i metryki wydajności we wszystkich funkcjach LLM.

## Zapisywanie i wskazówki

- Większość sekcji zapisuje się automatycznie chwilę po edycji — wskaźnik zapisywania i ewentualny błąd pojawiają się na dole sekcji;
- `MCP`, `DDNS` i `Pulpit zdalny` trzymają zmiany do wciśnięcia `Zapisz zmiany`: gdy zmiany czekają na zapis, obok przycisków pojawia się podpowiedź `Niezapisane zmiany`, a `Resetuj` je odrzuca;
- Otwarte z klienta mobilnego Web UI podąża za językiem i motywem aplikacji i ukrywa własne przełączniki języka i motywu.
