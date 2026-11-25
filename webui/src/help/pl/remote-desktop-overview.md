Pulpit zdalny pozwala oglądać i sterować ekranami serwera z klienta desktopowego w czasie rzeczywistym: pulpit maszyny jest streamowany do okna po Twojej stronie, z wstrzykiwaną myszą i klawiaturą — jakbyś siedział przed nim. Poza ekranami pulpitów emulatory Android i symulatory iOS można oglądać i prowadzić w oknach w ramce telefonu.

![Klient desktopowy — okno pulpitu zdalnego używane obok terminali](images/remote-desktop-overview-hero.webm)

## Co potrafi

- **Ekrany pulpitu** — każdy fizyczny wyświetlacz serwera streamuje do własnego, skalowalnego okna klienta;
- **Urządzenia / emulatory Android** — podłączone urządzenia i emulatory AV pojawiają się jako źródła, oglądalne i dotykalne w oknie w ramce telefonu, z obrotem i zdalnym uruchamianiem/zamykaniem emulatora;
- **Symulator iOS** (serwery macOS) — podgląd symulatora iOS Xcode w oknie w ramce telefonu z wyborem urządzenia i obrotem.

## Jak działa

Serwer przechwytuje ekran i koduje go konfigurowalnym koderem, klatkami, pułapem rozdzielczości i bitrate; klient dekoduje do unoszących się okien i odsyła zdarzenia wejścia istniejącym szyfrowanym połączeniem klient-serwer — bez dodatkowych portów.

Przygotowanie platformowe się różni: Windows może uruchomić serwer ekranu jako usługę systemową i wymaga wirtualnego sterownika ekranu na maszynach headless; Linux używa zgody Portal plus odblokowania sesji; macOS wymaga zgody na Nagrywanie ekranu. Zob. Konfigurację serwera; urządzenia Android omawia Urządzenia Android, a okna klienta — Korzystanie z klienta.
