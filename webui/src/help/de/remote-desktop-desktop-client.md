Das Desktop-Bildschirmfenster hat einen rahmenlosen Vollbildmodus für längere Arbeitssitzungen: Die Titelleiste verschwindet und das gestreamte Video füllt das gesamte Fenster. Diese Seite behandelt den Vollbildmodus und den schwebenden Menü-Button, der das Menü der Titelleiste vertritt; wie Bildschirmfenster geöffnet werden, steht in Client verwenden.

## Vollbildmodus

- Ein Doppelklick auf die Titelleiste schaltet Vollbild um (wie der Maximieren-Button); auch das Rechtsklick-Menü der Titelleiste hat einen Eintrag `Vollbild`;

  ![Die Titelleiste — Rechtsklickmenü mit dem Vollbild-Eintrag](images/remote-desktop-desktop-client-titlebar.jpg)

- Beim Eintreten spielt der schwebende Menü-Button (unten) eine kurze Aufmerksamkeits-Animation, damit Sie sehen, wo er sitzt;
- Beim Verlassen kehren Titelleiste und Rechtsklick-Menü zurück, und der Button verschwindet.

Im Vollbild wird jede physische Taste wie jede andere an den Remote-Host weitergeleitet: Es ist keine Tastenkombination lokal reserviert. Um das Vollbild zu verlassen, nutzen Sie den Eintrag `Vollbild beenden` im Menü des schwebenden Buttons.

## Der schwebende Menü-Button

Vollbild blendet die Titelleiste aus — und mit ihr das Rechtsklick-Menü — deshalb schwebt ein kleiner runder Button mit Hamburger-Symbol nahe der oberen Fenstermitte:

- Ein Klick öffnet dasselbe Menü, das die Titelleiste bot — Vollbild beenden, Tasten senden, Netzwerk-Bandbreite — direkt unter dem Button;
- Ziehen Sie ihn gedrückt an eine beliebige Stelle im Fenster;
- Im Leerlauf dimmt er auf 30 % Deckkraft ab, um das Video freizuhalten; Hover oder Drücken stellt die volle Deckkraft wieder her.

![Das Vollbildfenster — der schwebende Menü-Button mit dem darunter geöffneten Menü](images/remote-desktop-desktop-client-fab.jpg)

## Den verblassenden Button finden

Auf hellem Video kann der abgedimmte Button schwer wiederzufinden sein. Klicken Sie irgendwo im Fenster schnell **viermal** mit der linken Taste — vier Klicks innerhalb einer Sekunde —, dann spielt der Button seine Eintritts-Animation erneut ab: Eine Platte in Akzentfarbe mit weißem Symbol pulst dreimal, jeder Puls sendet einen sich ausbreitenden Ring aus, danach kehrt er in die abgedimmte Ruhe zurück.

Details:

- Klicks auf den Button selbst zählen nicht zum Burst (ebenso wenig Titelleiste oder ein geöffnetes Menü);
- Die vier Klicks werden unterdessen als normale Klicks an den Remote-Desktop weitergeleitet — zielen Sie auf eine leere Stelle des Desktops;
- Beim Eintritt ins Vollbild wird die Animation immer einmal abgespielt; die Geste dient also vor allem dazu, den Button später wiederzufinden.

![Die Aufdeck-Geste — vier schnelle Klicks lassen die Aufmerksamkeits-Animation des Buttons über dem Video ablaufen](images/remote-desktop-desktop-client-reveal.webp)
