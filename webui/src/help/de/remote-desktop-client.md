Der Desktop-Client öffnet Remote-Bildschirme über die SCREENS-Leiste oben im Debug-Panel. Jeder Bildschirm lebt in einem eigenen schwebenden Fenster, das Sie neben Terminals nutzen können.

## Die SCREENS-Leiste öffnen

1. Verbinden Sie sich mit dem Zielserver (direkt oder über ein Relay);
2. Klicken Sie in der linken Aktivitätsleiste auf den Tab `Debug` (das Werkzeugsymbol);
3. Die `SCREENS`-Leiste erscheint oben im Panel: eine Anzeige-Schaltfläche pro physischem Desktop-Bildschirm des Servers, gefolgt von den Umschaltern `iOS Simulator` und `Android Simulator`.

![Das Debug-Panel — die SCREENS-Leiste mit Anzeige-Schaltflächen und Simulator-Umschaltern](images/remote-desktop-client-screens.jpg)

## Desktop-Bildschirmfenster

Klicken Sie eine Anzeige-Schaltfläche, um das Fenster dieses Bildschirms zu öffnen:

- Das Fenster ist frei skalierbar (bis hinunter zu 800×600) und das Bild legt sich mit um;
- Multimonitor-Server erhalten ein Fenster pro Anzeige; mehrere können gleichzeitig offen sein;
- Beim Verbinden zeigt das Fenster einen Platzhalter „Desktop wird vorbereitet“; fehlt dem Server die Erfassungsberechtigung, zeigt das Fenster die Berechtigungsanleitung der Plattform direkt an;
- Erneutes Klicken auf den Leisten-Button oder Schließen des Fensters beendet das Betrachten (der aktive Zustand des Buttons bleibt mit dem Fenster synchron).

## iOS-Simulator-Fenster

Klicken Sie `iOS Simulator`, um ein fest dimensioniertes telefongeformtes Fenster zu öffnen (264×640 hochkant):

- Die Geräteauswahl oben wechselt das Telefon-Aussehen; das Fenster passt sich der gewählten Auflösung an;
- `Rotate` dreht hochkant/quer (524×380 querformatiges Fenster);
- Erfordert einen macOS-Server mit laufendem Xcode-iOS-Simulator.

## Android-Emulator-Fenster

`Android Simulator` funktioniert genauso: fester Telefonrahmen, Geräteauswahl, Drehen-Button. Streambare Quellen umfassen laufende AVD-Emulatoren und angeschlossene echte Geräte; ein AVD lässt sich vom Client auch auf Abruf starten oder herunterfahren (siehe Android-Geräte).

> Fenster, die einen localhost-Server streamen, werden automatisch von der Bilderfassung ausgeschlossen, um ein Rückkopfungsspiegel zu vermeiden; Fenster, die eine Remote-Maschine streamen, bleiben erfassbar, damit eine äußere Aufnahme sie weiterhin zeigt.
