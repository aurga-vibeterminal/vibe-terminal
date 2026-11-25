Eine Maschine mit dem Relay zu verbinden braucht zwei Felder: die Relay-Adresse und ihren Schlüssel. Was die Funktion ist und wie die Teile zusammenpassen, steht auf der vorigen Seite, dem Überblick.

## Den Relay-Server einrichten

1. Öffnen Sie auf der Maschine, die das Relay sein soll, den Bereich `Server`.
2. Stellen Sie sicher, dass ein `Secret Key` gesetzt ist — der Würfel-Button erzeugt einen. Derselbe Schlüssel koppelt Clients und geteilte Server.
3. Klappen Sie den `QR Code`-Block auf und wählen Sie `Network Interface` / `IP Version`, über die Clients und geteilte Server ihn erreichen sollen.

## Einen geteilten Server verbinden

Die Verbindungsdaten des Relays reisen über die Zwischenablage — kein Abtippen:

1. Klicken Sie in der Web UI des Relay-Servers auf den `QR code`. Das kopiert einen verschlüsselten Share-String (Adresse, Port, Secret Key) in die Zwischenablage.
2. Öffnen Sie die Web UI auf der zu teilenden Maschine, Bereich `Server`, und klicken Sie in das Feld `Relay Address`.
3. Der Browser blendet einen Dialog ein, der fragt, ob diese Website auf die Zwischenablage zugreifen darf — klicken Sie `Allow`:

![Browser — der Dialog zur Zwischenablagen-Berechtigung beim Fokussieren des Feldes](images/relay-allow-paste.jpg)

4. Mit erteilter Freigabe wird die Zwischenablage geprüft: Enthält sie den Share-String, erscheint neben dem Feld ein Einfügen-Button — klicken Sie ihn, um `Relay Address` und `Relay Secret Key` zu entschlüsseln und automatisch einzutragen:

![Web UI — Relay-Adresse und Relay-Secret-Key automatisch aus dem eingefügten Share-String gefüllt](images/relay-paste-share.jpg)

5. Prüfen Sie die Werte und klicken Sie `Apply` — das Speichern verbindet das Upstream-Relay neu.

![Web UI — die gefüllten Relay-Felder mit dem Apply-Button](images/relay-apply-share.jpg)

> Das Lesen der Zwischenablage erfordert einen sicheren Kontext (HTTPS oder localhost). Auf plain HTTP bleibt der Einfügen-Button verborgen; tragen Sie Relay-Adresse und Schlüssel von Hand ein.

Standardmäßig wird die Upstream-Verbindung verschlüsselt, aber nicht verifiziert. Um auch das Zertifikat des Relays festzunageln, siehe Strikte Verifizierung auf dem Relay unter Zertifikate & TLS.

## Desktop-Client

### Tabs geteilter Server

Tabs von Terminals auf einem geteilten Server tragen zwei Merkmale: einen farbigen Streifen am linken Rand des Tabs — eine Farbe pro Server — und das Remote-OS-Symbol (Windows / Apple / Linux) neben dem Statuspunkt. Lokale Tabs auf dem Relay-Server selbst haben keine Merkmale.

![Desktop-Client — Tab-Merkmale von Terminals geteilter Server](images/relay-tab-indicator.jpg)

### Terminal-Profile pro Server

Das Profil-Menü (Titelleiste) gruppiert Terminal-Profile nach Server: zuerst die eigenen Profile des verbundenen Relay-Servers, dann eine einklappbare Gruppe pro geteiltem Server, nach der Maschine benannt. Klappen Sie eine Gruppe auf und wählen Sie ein Profil, um ein Terminal auf diesem Server zu öffnen. Das Menü folgt der in der Titelleiste gewählten Tab-Gruppe.

![Desktop-Client — Profil-Menü mit Gruppen pro Server](images/relay-profile-menu.jpg)

### Gruppierung und getrennte Fenster

Der Gruppenwähler der Titelleiste listet die Tab-Gruppen aller Server; die Standardgruppe eines geteilten Servers erscheint als `Ungrouped (Servername)`. Das Untermenü jeder Gruppe listet ihre Tabs zum schnellen Wechseln und bietet `Move Group to New Window`, das die ganze Gruppe in ein eigenes Fenster reißt — nützlich, um ein Fenster oder einen Monitor einer Maschine zu widmen.

![Desktop-Client — Gruppenwähler-Menü mit der Aktion In-neues-Fenster-verschieben](images/relay-group-selector.jpg)
