Są dwa sposoby używania WSL w Windows: lekki dodaje powłokę WSL jako profil terminala, pełny instaluje vterm-server wewnątrz WSL. Pierwszy daje tylko powłokę; drugi odblokowuje zarządzanie plikami, panel Git, MCP i całą resztę.

## Sposób lekki: profil terminala WSL

W Windows **profile terminala WSL ładują się automatycznie**: przy starcie serwer wykrywa `wsl.exe` i wylicza zainstalowane dystrybucje, dodając po jednym wpisie profilu na dystrybucję do listy (np. `Ubuntu (WSL)`; gdy brak dystrybucji, pojawia się ogólny wpis `WSL`).

Aby go użyć, po prostu wybierz wpis WSL w menu profili na pasku tytułu klienta — dystrybucja startuje i stawia Cię prosto w jej terminalu, jak przy otwarciu lokalnego.

Ograniczenie: takie sesje działają na serwerze po stronie Windows — **eksplorator plików, panel Git, zmienne środowiskowe i podobne w kliencie nie działają** (operują semantyką systemu plików Windows i nie widzą linuksowego systemu plików w WSL).

![Menu profili terminala — automatycznie wypisane dystrybucje WSL](images/terminal-wsl-profile.jpg)

## Sposób pełny: instalacja vterm-server w WSL

Traktuj WSL jako osobną maszynę linuksową i zainstaluj bezgłowy vterm-server. Strona oferuje jedno_linijkowy skrypt — wklej go do terminala WSL:

```bash
curl -fsSL https://vterminal.dev/install.sh | bash
```

Skrypt automatycznie:

1. Wykrywa środowisko WSL/headless i pobiera **bezgłowy** vterm-server do `~/.local/vterm`;
2. Instaluje **usługę użytkownika systemd** i natychmiast ją uruchamia — działa przy każdym logowaniu (w tym starcie WSL), bez ręcznego startu;
3. Dodaje `~/.local/vterm` do PATH.

> Gdy systemd w WSL jest wyłączone, skrypt wskazuje dodanie `[boot]` `systemd=true` do `/etc/wsl.conf`, uruchomienie `wsl --shutdown` po stronie Windows i ponowne przepuszczenie instalatora.

## Otwarcie strony konfiguracji

W WSL:

```bash
vterm-server --open
```

Polecenie wypisuje adres Web UI (`http://localhost:<config-port>/`) i próbuje otworzyć przeglądarkę (w WSL przez `wslview`, domyślną przeglądarkę Windows). Dzięki przekazywaniu localhost WSL2 **otwarcie tego samego adresu w przeglądarce Windows** działa równie dobrze. Gdy serwer działa, wydrukowany adres zawiera jednorazowy token logowania (`?ticket=...`, wydany przez lokalne gniazdo), więc strona otwiera się już zalogowana.

## (Opcjonalnie) Udostępnienie serwera WSL w przekaźniku hosta

Jeśli maszyna Windows też uruchamia vterm-server (jako przekaźnik), serwer WSL może podpiąć się pod niego, by klient widział obie „maszyny" przez jedno połączenie lokalne:

1. W Web UI serwera Windows kliknij kod QR, aby skopiować ciąg udostępniania;
2. Otwórz Web UI serwera WSL (adres z poprzedniej sekcji), kliknij przycisk wklejania pojawiający się obok pola `Relay Address`, aby automatycznie wypełnić adres i klucz, a potem `Apply`;
3. Dopóki vterm-server Windows działa, klient widzi terminale, pliki i Git serwera WSL pod lokalnym przekaźnikiem.

> Pod domyślnym NAT-em WSL2 WSL dociera do hosta Windows adresem hosta widzianym z WSL (brama domyślna — nameserver w `cat /etc/resolv.conf`); przy sieci lustrzanej działa `localhost`. Szczegóły przekaźnika — w rozdziale Relay.
