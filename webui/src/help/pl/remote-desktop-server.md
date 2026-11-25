Wszystko mieszka w sekcji `Remote Desktop` Web UI; strona pokazuje ustawienia właściwe dla platformy serwera.

## Główny przełącznik

`Enable Remote Desktop` — raz włączony, podłączeni zdalni klienci mogą oglądać pulpit tej maszyny i wysyłać wejście.

## Ustawienia platform

**Windows**

- `Windows Service` — rejestruje serwer ekranu jako samostartującą usługę systemową, by pulpit zdalny przetrwał restarty bez zalogowanej sesji. `Install` / `Uninstall` wywołuje na serwerze monit UAC (przycisk pokazuje postęp podczas czekania); plakietka statusu odróżnia działa / zainstalowana / niezainstalowana.
- `Maszyny headless (bez monitora)` — bez podłączonego (lub obudzonego) ekranu Windows nie utrzymuje przechwytywalnego pulpitu. Strona linkuje pobranie otwartoźródłowego Virtual Display Driver (ARM64 / x64 wybierane automatycznie); zainstaluj go, by zawsze istniał wirtualny pulpit.

![Web UI — sekcja Remote Desktop pod Windows (karty usługi i headless)](images/remote-desktop-server-windows.jpg)

**Linux**

- `Unlock on Connect` — automatycznie odblokowuje zablokowaną sesję, gdy stream się rozpoczyna (gdzie środowisko to wspiera);
- przechwytywanie ekranu idzie przez Portal; `Reset Permission` na karcie `Screen Recording Permission` usuwa zapisaną zgodę Portal, by wybrać inny ekran.
- `Auto-start on Login` (w sekcji `Serwer`) — na hostach z menedżerem wyświetlania serwer startuje razem z sesją graficzną, więc przechwytywanie dziedziczy pełne środowisko pulpitu; na hostach headless startuje natomiast przy rozruchu, przed jakimkolwiek logowaniem. By dotrzeć do pulpitu, gdy nikt nie zalogował się lokalnie, włącz też automatyczne logowanie w menedżerze wyświetlania (GDM, SDDM, …); dla bezpieczeństwa automatyczna blokada ekranu może pozostać włączona — sesja i serwer działają dalej, gdy jest zablokowana, a `Unlock on Connect` (lub `loginctl unlock-session`) otwiera sesję zdalnie.
![Web UI — karta Screen Recording Permission (Linux)](images/remote-desktop-server-permission.jpg)


**macOS**

- `Screen Recording Permission` — system wymaga jawnej zgody na przechwytywanie. Kliknij `Grant Permission`, by otworzyć okno systemowe; `Reset Permission` wyprowadza z zaciętego stanu.

## Narzędzia streamowania ekranu (obsługa Androida)

`Android SDK directory` i `Android Studio directory` lokalizują adb i emulator; zostaw puste dla autodetekcji. Po ustawieniu strona pokazuje rozwiązany ścieżkę i jej pochodzenie (auto / env / override); `Reset to Auto-Detect` przywraca domyślne. Co odblokowują — zob. Urządzenia Android.

## Kodowanie streamu

- `Encoder` — Auto albo konkretny backend; serwer wypisuje użyteczne kodery i szarzy niedostępne z powodem;
- `Frame rate` — 60 / 30 / 15 kl./s;
- `Max resolution` — pułap po stronie przechwytywania (4K / 2.5K / 2K);
- `Bitrate` — `Auto` skaluje powierzchniowo z rozdzielczością (bazowa do 1080p, ~4× przy 4K, z limitem 40 Mbps); stała wartość działa natychmiast na bieżących streamach, a koder / fps / rozdzielczość — na streamach startowanych później;
- stopka karty pokazuje aktualnie obowiązujący koder / fps / rozdzielczość / bitrate.

![Web UI — ustawienia kodowania streamu](images/remote-desktop-server-stream.jpg)

Kliknij `Save`, by zastosować zmiany.
