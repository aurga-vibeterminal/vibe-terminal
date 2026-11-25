De desktopclient opent externe schermen vanuit de SCREENS-balk bovenaan het Debug-paneel. Elk scherm leeft in zijn eigen zwevende venster dat je naast terminals kunt gebruiken.

## De SCREENS-balk openen

1. Verbind met de doelserver (rechtstreeks of via een relay);
2. klik op het tabblad `Debug` in de linkeractiviteitenbalk (het gereedschapspictogram);
3. de balk `SCREENS` verschijnt bovenaan het paneel: één beeldschermknop per fysiek bureaubladscherm op de server, gevolgd door de schakelaars `iOS Simulator` en `Android Simulator`.

![Het Debug-paneel — de SCREENS-balk met beeldschermknoppen en simulator-schakelaars](images/remote-desktop-client-screens.jpg)

## Bureaubladschermvensters

Klik op een beeldschermknop om het venster van dat scherm te openen:

- het venster is vrij schaalbaar (tot 800×600) en het beeld herschikt zich mee;
- multi-monitor-servers krijgen één venster per scherm; er kunnen er meerdere tegelijk open zijn;
- tijdens het verbinden toont het venster een tijdelijke aanduiding "bureaublad voorbereiden"; mist de server vastlegtoestemming, dan toont het venster direct de toestemmingsinstructies van het platform;
- klik opnieuw op de balkknop of sluit het venster om te stoppen met kijken (de actieve toestand van de knop blijft synchroon met het venster).

## iOS-simulatorvenster

Klik `iOS Simulator` om een vast omrand telefoonvenster te openen (264×640 staand):

- de apparaatkeuze bovenin wisselt het telefoonuiterlijk; het venster past zich aan de gekozen resolutie aan;
- `Rotate` wisselt staand/liggend (524×380 liggend venster);
- vereist een macOS-server met de Xcode iOS-simulator draaiende.

## Android-emulatorvenster

`Android Simulator` werkt hetzelfde: vaste telefoonlijst, apparaatkeuze, roteerknop. Streambare bronnen omvatten draaiende AVD-emulators en verbonden echte apparaten; een AVD kan ook op verzoek vanaf de client worden opgestart of afgesloten (zie Android-apparaten).

> Vensters die een localhost-server streamen worden automatisch van schermvastlegging uitgesloten om een terugkoppelingsspiegel te vermijden; vensters die een externe machine streamen blijven vastlegbaar zodat een buitenste opname ze nog toont.
