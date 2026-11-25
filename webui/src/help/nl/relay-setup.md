Een machine met de relay verbinden kost twee velden: het relay-adres en zijn sleutel. Zie de vorige pagina, Overzicht, voor wat de functie is en hoe de stukken in elkaar passen.

## De relay-server instellen

1. Open de sectie `Server` op de machine die als relay zal fungeren.
2. Zorg dat een `Secret Key` is ingesteld — de dobbelsteenknop genereert er een. Dezelfde sleutel koppelt zowel clients als gedeelde servers.
3. Vouw het `QR Code`-blok uit en kies de `Network Interface` / `IP Version` die clients en gedeelde servers moeten gebruiken om hem te bereiken.

## Een gedeelde server verbinden

De verbindingsgegevens van de relay reizen op het klembord — geen overtypen:

1. Klik in de Web UI van de relay-server op de `QR code`. Dit kopieert een versleutelde deeltekenreeks (adres, poort, geheime sleutel) naar het klembord.
2. Open de Web UI op de te delen machine, sectie `Server`, en klik in het veld `Relay Address`.
3. De browser toont een dialoog die vraagt of deze site toegang tot het klembord krijgt — klik `Allow`:

![Browser — het klembordtoegangs-toestemmingsvenster bij het focussen van het veld](images/relay-allow-paste.jpg)

4. Met toestemming wordt het klembord gecontroleerd: bevat het de deeltekenreeks, dan verschijnt er een plakknop naast het veld — klik die om te ontsleutelen en `Relay Address` en `Relay Secret Key` automatisch in te vullen:

![Web UI — relay-adres en relay-geheime sleutel automatisch gevuld vanuit de geplakte deeltekenreeks](images/relay-paste-share.jpg)

5. Controleer de waarden en klik `Apply` — opslaan maakt opnieuw verbinding met de stroomopwaartse relay.

![Web UI — de gevulde relay-velden met de Apply-knop](images/relay-apply-share.jpg)

> Het klembord lezen vereist een veilige context (HTTPS of localhost). Op gewone HTTP blijft de plakknop verborgen; typ het relay-adres en de sleutel met de hand in.

Standaard wordt de upstream-verbinding versleuteld maar niet geverifieerd. Om ook het certificaat van de relay vast te pinnen, zie Stricte verificatie op de relay onder Certificaten & TLS.

## Desktopclient

### Tabbladen van gedeelde servers

Tabbladen van terminals die op een gedeelde server draaien dragen twee markeringen: een gekleurde streep aan de linkerrand van het tabblad — één kleur per server — en het externe OS-pictogram (Windows / Apple / Linux) naast de statusstip. Lokale tabbladen op de relay-server zelf hebben geen markeringen.

![Desktopclient — tabbladmarkeringen van terminals op gedeelde servers](images/relay-tab-indicator.jpg)

### Terminalprofielen per server

Het profielmenu (titelbalk) groepeert terminalprofielen per server: eerst die van de verbonden relay-server zelf, daarna één opvouwbare groep per gedeelde server, genoemd naar de machine. Vouw een groep uit en kies een profiel om een terminal op die server te openen. Het menu volgt de in de titelbalk gekozen tabbladgroep.

![Desktopclient — profielmenu met groepen per server](images/relay-profile-menu.jpg)

### Groeperen en afzonderlijke vensters

De groepskezer van de titelbalk somt de tabbladgroepen van alle servers op; de standaardgroep van een gedeelde server verschijnt als `Ungrouped (Servernaam)`. Het submenu van elke groep somt zijn tabbladen voor snel wisselen en biedt `Move Group to New Window`, dat de hele groep naar een eigen venster scheurt — handig om een venster of monitor aan één machine te wijden.

![Desktopclient — groepskeuzermenu met de actie verplaatsen naar nieuw venster](images/relay-group-selector.jpg)
