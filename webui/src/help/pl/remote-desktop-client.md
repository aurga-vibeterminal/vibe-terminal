Klient desktopowy otwiera zdalne ekrany z paska SCREENS u góry panelu Debug. Każdy ekran żyje we własnym unoszącym się oknie, używalnym obok terminali.

## Otwarcie paska SCREENS

1. Połącz się z serwerem docelowym (bezpośrednio albo przez przekaźnik);
2. kliknij kartę `Debug` w lewym pasku aktywności (ikona narzędzi);
3. pasek `SCREENS` pojawia się u góry panelu: po jednym przycisku ekranu na każdy fizyczny pulpit serwera, a za nimi przełączniki `iOS Simulator` i `Android Simulator`.

![Panel Debug — pasek SCREENS z przyciskami ekranów i przełącznikami symulatorów](images/remote-desktop-client-screens.jpg)

## Okna ekranów pulpitu

Kliknij przycisk ekranu, by otworzyć okno tego ekranu:

- okno swobodnie skaluje (do 800×600), a obraz przekłada się z nim;
- serwery wielomonitorowe dostają okno na ekran; kilka może być otwartych naraz;
- podczas łączenia okno pokazuje zastępczkę „przygotowanie pulpitu"; gdy serwerowi brakuje zgody na przechwytywanie, okno od razu pokazuje instrukcje uprawnień platformy;
- ponowne kliknięcie przycisku paska albo zamknięcie okna przerywa oglądanie (stan aktywny przycisku pozostaje zsynchronizowany z oknem).

## Okno symulatora iOS

Kliknij `iOS Simulator`, by otworzyć stałorozmiarowe okno w ramce telefonu (264×640 pion):

- wybór urządzenia u góry zmienia wygląd telefonu; okno dopasowuje się do wybranej rozdzielczości;
- `Rotate` przełącza pion/poziom (okno 524×380 w poziomie);
- wymaga serwera macOS z uruchomionym symulatorem iOS Xcode.

## Okno emulatora Androida

`Android Simulator` działa tak samo: stała ramka telefonu, wybór urządzenia, przycisk obrotu. Streamowalne źródła obejmują działające emulatory AVD i podłączone prawdziwe urządzenia; AVD można też uruchomić lub zamknąć na życzenie z klienta (zob. Urządzenia Android).

> Okna streamujące serwer localhost są automatycznie wykluczane z przechwytywania ekranu, by uniknąć lustra sprzężenia; okna streamujące maszynę zdalną pozostają przechwytywalne, by zewnętrzne nagranie nadal je pokazywało.
