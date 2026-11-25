Poza ekranami pulpitu pulpit zdalny może streamować urządzenia Android — prawdziwe telefony przez adb i emulatory AVD. Klient pokazuje je w oknie w ramce telefonu z wejściem dotykowym, obrotem i zdalnym sterowaniem cyklem życia emulatora.

## Wymagania wstępne

Serwer musi odnaleźć zestaw narzędzi Androida:

- **Android SDK** (dostarcza `adb`) — wymagany i dla urządzeń, i emulatorów;
- **Android Studio** — wymagany do uruchamiania emulatorów AVD.

Oba są domyślnie autodetekowane. Gdy detekcja zawiedzie albo narzędzia mieszkają poza standardowymi miejscami, ustaw `Android SDK directory` / `Android Studio directory` w `Remote Desktop` → `Screen streaming tools`; zmiany działają natychmiast (na gorąco, bez restartu serwera).

## Źródła urządzeń

Z ustawionymi ścieżkami serwer automatycznie wylicza dostępne źródła Androida:

- **Podłączone prawdziwe urządzenia** — podepnij przez USB z włączonym debugowaniem USB; wszystko, co widzi `adb devices`, może streamować;
- **Emulatory AVD** — działające emulatory pojawiają się same; wybrany AVD można też uruchomić na życzenie z klienta (bez ręcznego startowania najpierw) i zdalnie zamknąć.

Każde źródło dostaje własną pozycję w pasku SCREENS klienta.

## Orientacja

Przycisk obrotu okna emulatora ustawia systemowy `user_rotation` przez `adb` — zmienia się własny kierunek wyświetlacza urządzenia, a okno dopasowuje się do nowej rozdzielczości, radząc sobie i z pionem, i poziomem.

## Strona klienta

Otwórz kartę `Debug` w lewym panelu klienta desktopowego i kliknij przycisk `Android Simulator` w pasku SCREENS (pojawia się tylko, gdy serwer udostępnia źródła Androida), by otworzyć okno emulatora; wybór urządzenia zmienia wygląd ramki telefonu, a `Rotate` odwraca orientację. Zob. Korzystanie z klienta.

> Wstrzykiwanie dotyku na prawdziwych urządzeniach używa kanału wejścia adb (Android 4.4+); niektóre ROM-y producentów je ograniczają — wtedy urządzenie jest tylko do podglądu.
