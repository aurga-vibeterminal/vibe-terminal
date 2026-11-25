Verbinden Sie einen Client über den Pairing-QR-Code der Web UI mit diesem Server — oder durch handisches Eintippen. Alles, was ein Client braucht (Adresse, Port, Secret Key), steht im Server-Bereich.

## Server vorbereiten

- Öffnen Sie den Bereich `Server` und stellen Sie sicher, dass ein `Secret Key` gesetzt ist; klicken Sie auf den Würfel-Button, um einen zu erzeugen. Ein neu erzeugter Schlüssel macht alle gekoppelten Clients ungültig — diese müssen sich neu koppeln.
- Klappen Sie den `QR Code`-Block auf und wählen Sie `Network Interface` und `IP Version` (IPv4 / IPv6), über die andere Maschinen diesen Server erreichen sollen.
- Ist eine DDNS-Domain konfiguriert, können Sie `Connection Type` auf `DDNS Domain` umstellen, damit der QR-Code die Domain statt einer IP-Adresse einbettet.
- Die `Pairing Preview` zeigt die exakte Verbindungs-URL (`wss://host:port`), die der Client anwählt.
- Ein Klick auf den `QR code` kopiert einen verschlüsselten Share-String in die Zwischenablage — Clients nutzen ihn zum Auto-Ausfüllen mit einem Tipp.

## Desktop-Client

- Klicken Sie in der Titelleiste auf den Server-Namen-Button und wählen Sie im Menü `Servers`, um den Server-Manager-Dialog zu öffnen.

![Desktop-Client — das Server-Menü, geöffnet über den Server-Namen-Button](images/start-connect-server-menu.jpg)

- Klicken Sie neben der Serverliste auf `+`, um das Add-Server-Formular zu starten.
- Liegt der Share-String in der Zwischenablage, erscheint oben rechts im Formular ein Zwischenablage-Symbol — klicken Sie es an, um den String zu entschlüsseln und `Title`, `Address` und `Secret Key` automatisch einzutragen.
- Andernfalls füllen Sie die Felder von Hand aus: ein Anzeigename, die Verbindungs-URL (`wss://host:port`, IPv6-Adressen in Klammern) und der Secret Key.
- Klicken Sie auf `Save` und wählen Sie den neuen Server, um eine Sitzung zu öffnen.

![Desktop-Client — Server-Manager mit dem Add-Server-Formular](images/start-connect-add-server.jpg)
