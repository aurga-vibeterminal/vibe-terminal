De relay lost één probleem op: de meeste machines zitten achter NAT of dynamische IP's, dus clients kunnen ze niet elk afzonderlijk rechtstreeks bellen. Laat één publiek bereikbare machine optreden als relay-toegangspunt; elke andere machine belt naar buiten naar hem. Een client onderhoudt dan één verbinding met de relay en bereikt daardoor de terminals van elke gedeelde server.

## Architectuur

![Relay-architectuur — elke client houdt één verbinding met de relay; gedeelde servers bellen de relay stroomopwaarts](diagram/relay)

- **Relay-server** — het publieke toegangspunt waarmee clients en gedeelde servers verbinden. Laat zijn `Relay Address` leeg.
- **Gedeelde server** — een machine waarvan `Relay Address` naar de relay wijst. Hij belt stroomopwaarts en deelt zijn terminals erdoorheen. De gekleurde streep op elke gedeelde node in het diagram spiegelt de streep die de desktopclient op de tabbladen van die server schildert — één kleur per server.
- Verbindingen ontstaan altijd van binnenuit: gedeelde servers bellen de relay, dus poortforwarding of een publiek IP is op hen niet nodig.

## Wat de client ziet

Eenmaal met de relay verbonden gedraagt elke gedeelde server zich als een lokale machine:

- Terminaltabbladen dragen een gekleurde streep en het externe OS-pictogram, waarmee de machine in één oogopslag herkenbaar is (zie de Installatiehandleiding);
- het profielmenu groepeert terminals per server — open direct een terminal op elke machine;
- tabbladgroepen kunnen naar hun eigen venster worden losgetrokken, zodat een venster aan één machine wordt gewijd.

## Beveiligingsmodel

Clients en gedeelde servers koppelen met dezelfde geheime sleutel; verbindingen worden door de relay getunneld. De sleutel reist in de koppel-QR-code / deeltekenreeks, nooit in platte tekst.

Stapsgewijze instructies staan op de volgende pagina, Installatiehandleiding.
