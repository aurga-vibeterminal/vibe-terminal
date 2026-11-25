Remote Desktop lässt Sie die Bildschirme des Servers vom Desktop-Client aus in Echtzeit beobachten und steuern: Der Desktop der Maschine wird in ein Fenster auf Ihrer Seite gestreamt, mit Injektion Ihrer Maus und Tastatur — als säßen Sie davor. Über Desktop-Bildschirme hinaus lassen sich Android-Emulatoren und iOS-Simulatoren in telefongeformten Fenstern betrachten und steuern.

![Desktop-Client — ein Remote-Desktop-Fenster neben Terminals genutzt](images/remote-desktop-overview-hero.webm)

## Was es kann

- **Desktop-Bildschirme** — jede physische Anzeige des Servers streamt in ein eigenes, skalierbares Client-Fenster;
- **Android-Geräte / Emulatoren** — angeschlossene Geräte und AVD-Emulatoren erscheinen als Stream-Quellen, in einem telefongeformten Fenster betracht- und berührbar, mit Drehung und ferngesteuertem Emulator-Start/Stopp;
- **iOS-Simulator** (macOS-Server) — den Xcode-iOS-Simulator in einem telefongeformten Fenster mit Geräteauswahl und Drehung betrachten.

## Wie es funktioniert

Der Server erfasst den Bildschirm und kodiert ihn mit konfigurierbarem Encoder, Bildrate, Auflösungsobergrenze und Bitrate; der Client dekodiert in schwebende Fenster und sendet Eingabeereignisse über die bestehende verschlüsselte Client-Server-Verbindung zurück — keine zusätzlichen Ports.

Die Plattformvorbereitung unterscheidet sich: Windows kann den Bildschirmserver als Systemdienst laufen lassen und braucht auf Headless-Maschinen einen virtuellen Anzeigetreiber; Linux nutzt eine Portal-Zustimmung plus Sitzungs-Entsperrung; macOS verlangt die Berechtigung zur Bildschirmaufnahme. Siehe Server-Setup; Android-Geräte behandelt Android-Geräte, die Client-Fenster Client verwenden.
