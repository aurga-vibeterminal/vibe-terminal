## De browser waarschuwt „Uw verbinding is niet privé” — is dat normaal?

Ja. Doorgestuurde poorten, de Web UI en clientverbindingen delen hetzelfde zelfondertekende servercertificaat; de versleuteling zelf is niet aangetast. Importeer het certificaat van de server in de vertrouwde opslag van je apparaat om de waarschuwing definitief uit te schakelen.

## Is Vibe Terminal gratis?

Ja — de desktopclient/server en de headless-server zijn gratis. Download en installeer op zo veel machines als je wilt.

## Waar gaan mijn gegevens naartoe?

Nergens waar je ze niet zelf neerzet. Sessies, inloggegevens (enc-v2:) en spraakherkenning blijven op je eigen machines; koppelingssleutels reizen nooit als platte tekst.

## Kan ik een doorgestuurde dienst veilig blootstellen?

Ja. De ingebouwde shares vereisen een token: de links die de client genereert voor een `Host with HTTP server`-share of de Settings-pagina bevatten een toegangstoken, en een verbinding zonder geldig token en zonder geldige sessie wordt verbroken voordat er ook maar iets wordt geserveerd. Bij het eerste bezoek wordt het token ingewisseld voor een beveiligde sessiecookie die alleen voor die ene dienst geldt, en verdwijnt het uit de URL — een Settings-ticket werkt precies één keer en verloopt binnen een minuut, en een share stoppen of herstarten maakt eerdere links en sessies ongeldig. Behandel de volledige link als een toegangsbewijs en geef hem alleen aan mensen die je vertrouwt. Een forward naar je eigen dienst voegt TLS met het servercertificaat toe, maar geen authenticatie — houd authenticatie voor gevoelige diensten en breek een forward direct af via het paneel poort forwarding zodra je klaar bent.
