Screenshots en de terminal ontmoeten elkaar in één flow: met één globale sneltoets grijp je elk gebied van het bureaublad, je voorziet het van aantekeningen en kopieert of bewaart het — plak daarna de klembordafbeelding rechtstreeks in een terminal, waar ze landt als bestandspad dat het draaiende programma (een code-agent incluis) kan openen en gebruiken. Beide helften leven in de desktopclient.

## Een screenshot maken

- Druk overal op `Shift`+`Alt`+`A`: de sneltoets is globaal, hij gaat af ongeacht welke app de focus heeft zolang de desktopclient draait (op macOS drukt u `Shift`+`Command`+`A`);
- Elk scherm dimt onder een vastlegoverlay. Sleep om het selectiegebied te tekenen — het mag over meerdere schermen lopen — sleep daarna binnen het gebied om het te verplaatsen, of trek aan de handvatten om het te vergroten of verkleinen; een badge boven de selectie toont de grootte in pixels;
- Druk op elk moment op `Esc` om te annuleren.

![De vastlegoverlay — een selectiegebied met groottebadge en annotatiewerkbalk](images/terminal-images-capture.jpg)

## Annoteren

De werkbalk onder de selectie draagt links de annotatiehulpmiddelen (rechthoek en ellips), in het midden zes kleuren en rechts drie actieknoppen — annuleren (`✕`), opslaan (downloadpijl) en kopiëren (`✓`).

- Kies een hulpmiddel en een kleur, sleep dan binnen de selectie om te tekenen; streken worden op de selectie bijgesneden;
- `Ctrl`+`Z` maakt de laatste aantekening ongedaan, `Ctrl`+`R` herstelt haar;
- Een nieuw gebied buiten de huidige selectie slepen verwijdert haar — aantekeningen inbegrepen — en begint een nieuwe.

## Kopiëren of opslaan

- `Kopiëren` (`✓`) legt de geannoteerde vastlegging op het klembord — klaar om in een terminal (onder), een chat of elke andere app geplakt te worden;
- `Opslaan` (downloadpijl) schrijft via een opslagdialoog een PNG; de bestandsnaam is standaard `vterm-screenshot-<timestamp>.png`.

## Een afbeelding in een terminal plakken

- Kopieer een afbeelding naar het klembord — uit de screenshot-tool hierboven, een browser of de eigen vastlegging van het besturingssysteem;
- Rechtsklik op de terminal en kies `Paste Image`; de invoer verschijnt alleen als het klembord werkelijk een afbeelding bevat;
- De afbeelding wordt naar de server van de sessie geüpload en daar als tijdelijk bestand bewaard; het pad tussen aanhalingstekens wordt daarna op de cursor van de terminal ingevoegd, precies alsof je het typte — gewikkeld in bracketed-paste-markers wanneer de draaiende app die ondersteunt, zodat shells, editors en agents het netjes ontvangen;
- Voor het programma is het gewoon tekst: een code-agent zoals Claude Code of opencode opent de afbeelding via het pad, en gewone shellopdrachten kunnen hem als argument gebruiken.

![Terminal rechtsklik — de Paste Image-invoer](images/terminal-images-paste.jpg)

## Kanttekeningen

- Het ingevoegde pad leeft op de server van de sessie: bij een externe of WSL-server wordt het tijdelijke bestand op die machine gemaakt, precies waar de programma's die in de terminal draaien het kunnen bereiken;
- Gecombineerd met `Context toevoegen aan agents` is dit de snelste manier om een agent iets van je scherm te tonen — vastleggen, plakken, en de agent houdt de afbeelding als bestand;
- Onder Linux werkt de vastlegging onder X11; native Wayland-sessies ondersteunen noch de globale sneltoets noch de interactieve overlay, en als een andere X11-app `Shift`+`Alt`+`A` al vastheeft, kan de sneltoets niet worden geregistreerd.
