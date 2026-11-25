Das Relay löst ein Problem: Die meisten Maschinen sitzen hinter NAT oder dynamischen IPs, sodass Clients sie nicht einzeln direkt anwählen können. Lassen Sie eine öffentlich erreichbare Maschine als Relay-Eingangspunkt fungieren; jede andere Maschine wählt nach außen zu ihm. Ein Client hält dann eine einzige Verbindung zum Relay und erreicht darüber die Terminals jedes geteilten Servers.

## Architektur

![Relay-Architektur — jeder Client hält eine Verbindung zum Relay; geteilte Server wählen ins Relay nach oben](diagram/relay)

- **Relay-Server** — der öffentliche Eingangspunkt, mit dem sich Clients und geteilte Server verbinden. Lassen Sie seine `Relay Address` leer.
- **Geteilter Server** — eine Maschine, deren `Relay Address` auf das Relay zeigt. Sie wählt nach oben und teilt ihre Terminals durch das Relay. Der farbige Streifen an jedem geteilten Knoten im Diagramm spiegelt den Streifen, den der Desktop-Client auf den Tabs dieses Servers malt — eine Farbe pro Server.
- Verbindungen entstehen immer von innen: geteilte Server wählen das Relay an, also braucht es auf ihnen weder Portweiterleitung noch eine öffentliche IP.

## Was der Client sieht

Einmal mit dem Relay verbunden, verhält sich jeder geteilte Server wie eine lokale Maschine:

- Terminal-Tabs tragen einen farbigen Streifen und das Remote-OS-Symbol und identifizieren die Maschine auf einen Blick (siehe Einrichtungsanleitung);
- Das Profil-Menü gruppiert Terminals pro Server — öffnen Sie ein Terminal auf einer beliebigen Maschine direkt;
- Tab-Gruppen lassen sich in eigene Fenster reißen und so einem Gerät widmen.

## Sicherheitsmodell

Clients und geteilte Server koppeln sich mit demselben Secret Key; Verbindungen werden durch das Relay getunnelt. Der Schlüssel reist im Pairing-QR-Code / Share-String, nie im Klartext.

Schritt-für-Schritt-Anleitungen stehen auf der nächsten Seite, Einrichtungsanleitung.
