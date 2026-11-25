Vibe Terminal to zdalnie zarządzany hub terminali do tworzenia oprogramowania wbudowanego i serwerowego. Jeden **serwer** działa na każdej maszynie, na której pracujesz — maszynie deweloperskiej, serwerze budowania, płytce embedded — a **klienci** łączą się z nim z aplikacji desktopowej lub wiersza poleceń. Ten przewodnik obejmuje obie strony: Web UI, które konfiguruje serwer, oraz klientów, którzy z nim pracują.

## Jak to do siebie pasuje

- Wszystko działa na **serwerze** — sesje terminali żyją tam, więc zamknięcie klienta, utrata sieci czy przestawienie okien nigdy nie przerywa uruchomionego polecenia; po ponownym podłączeniu kontynuujesz od miejsca, w którym się przerwało;
- Klienci nawiązują z serwerem jedno szyfrowane połączenie WebSocket (`wss://host:port`; `wss://localhost:5153` dla serwera lokalnego) i uwierzytelniają się jego **kluczem tajnym**; parowanie przekazuje klucz przez kod QR lub zaszyfrowany ciąg udostępniania (zob. Łączenie klienta);
- **Web UI** to własna strona konfiguracyjna serwera — profile, dodatki, dostawcy LLM, przekaźnik, przekierowanie portów i więcej. To nie jest terminal: codzienna praca toczy się w kliencie desktopowym lub `vtermc`.

## Komponenty

- **Serwer** — działa na maszynie docelowej; utrzymuje sesje terminali i udostępnia podłączonym klientom pliki, Gita, pulpit zdalny oraz funkcje głosowe i AI;
- **Klient desktopowy** — natywna przestrzeń robocza: terminale z podziałami, grupami kart i odłączonymi oknami, eksplorator i edytor plików, narzędzia Git, wprowadzanie głosowe, historia rozmów agenta kodu, pulpit zdalny i zdalne debugowanie;
- **`vtermc` CLI** — uruchamiaj i podłączaj sesje z dowolnego zwykłego terminala, przez SSH albo na maszynie headless; współdzieli listę serwerów z klientem desktopowym (zob. Klient wiersza poleceń);
- **Web UI** — ustawienia samego serwera w przeglądarce, pogrupowane w sekcje na lewym pasku bocznym (zob. Korzystanie z Web UI).

## Ikona zasobnika

Na maszynach z sesją desktopową działający serwer pokazuje ikonę w zasobniku:

![Menu ikony zasobnika — `Open` wysuwa działającego klienta na pierwszy plan albo go uruchamia, `Settings` otwiera Web UI, a pozycja głosowa uruchamia lub zatrzymuje serwer rozpoznawania mowy](images/start-tray-menu.jpg)

- **Open** — wysuwa działającego klienta desktopowego na pierwszy plan albo uruchamia go, gdy żaden nie działa (podwójne kliknięcie ikony robi to samo);
- **Settings** — otwiera Web UI serwera w przeglądarce (zob. Korzystanie z Web UI);
- **Start/Stop Vibe Voice** — uruchamia lub zatrzymuje serwer rozpoznawania mowy bez otwierania Web UI, powielając przyciski Start/Stop sekcji `Głos`; pojawia się, gdy rozpoznawanie mowy jest włączone, a etykieta odzwierciedla stan (np. `Stop Vibe Voice (model on device)`, gdy model jest załadowany);
- **Exit** — zamyka serwer.

Gdy katalogi są publikowane przez HTTP, pozycja `HTTP Servers` pozwala zatrzymać każde udostępnienie jednym kliknięciem (zob. Publikacja przez HTTP).

## Mapa funkcji

Reszta tego przewodnika odzwierciedla rozdziały paska bocznego:

- **Terminale** — tworzenie, dzielenie i organizowanie terminali; profile, WSL i port szeregowy UART; szybkie polecenia i zmienne środowiskowe; sterowanie terminalami z zewnętrznych agentów przez MCP; przeglądanie rozmów agenta kodu; automatyczne potwierdzanie interaktywnych monitów;
- **Pliki** — ostatnie projekty, serwerowy eksplorator plików, wysyłanie i pobieranie plików oraz publikacja katalogu przez HTTP (zob. Eksplorator plików, Przesyłanie plików i Publikacja przez HTTP);
- **Git** — indeksowanie, commity z pomocą AI, worktree, interaktywny rebase, rozwiązywanie konfliktów i push;
- **Pulpit zdalny i symulatory** — podgląd i sterowanie ekranami serwera, urządzeniami i emulatorami Android oraz symulatorem iOS;
- **Wprowadzanie głosowe** — serwerowe rozpoznawanie mowy z dyktowaniem na przycisk i szlifem LLM;
- **Przekaźnik (Relay)** — docieranie do maszyn za NAT-em lub z dynamicznymi IP przez jeden publiczny punkt wejścia;
- **Dostęp zdalny** — domeny DDNS i przekierowanie portów TLS dla usług działających na serwerze;
- **Funkcje AI** — dostawcy LLM jako dodatki, przypisywani per funkcja, plus integracja MCP (zob. LLM i MCP).

## Od czego zacząć

1. Otwórz w Web UI sekcję `Server`, ustaw `Secret Key` i sparuj klienta — strona Łączenie klienta prowadzi przez konfigurację kodem QR i ręczną;
2. Przejrzyj sekcje Web UI, by skonfigurować profile, dodatki i funkcje — zob. Korzystanie z Web UI;
3. Otwórz terminal na serwerze i sięgaj do powyższych rozdziałów według potrzeb.
