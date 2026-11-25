Vertaling zet tekst die je in de client tegenkomt om in de taal van jouw keuze: antwoorden van de AI-chat, commitberichten, codecommentaren of elke selectie in de code-editor of een diff. Het werk doet een vertaalmodel dat op de server is geconfigureerd — de client stuurt alleen de tekst, rendert het resultaat en cachet het, zodat je tussen origineel en vertaling kunt wisselen.

## Waar vertaling beschikbaar is

- **Copilot-paneel** — elke berichtkop (je eigen berichten, antwoorden van het model en `Denkt na...`-blokken) heeft een vertaalicoon. One klik vervangt de berichttekst door de vertaling; nog een klik herstelt het origineel;
- **Commitbericht-concept** — het commitvak in het Git-paneel krijgt een vertaalknop zodra je concept meerdere regels beslaat. Het vertaalt het hele concept ter plekke, en nog een klik brengt je originele formulering terug;
- **Commitdetail-popup** — over een commit in de Git-geschiedenis hangen opent een detail-popup; het vertaalicoon naast de hash wisselt het commitbericht naar jouw taal;
- **Code-editor en diff-weergave** — selecteer wat dan ook (blokselecties tellen ook), klik met de rechtermuisknop en kies `Vertalen`. Een klein globe-badge markeert de vertaalde regels in de goot, en een kaart met de vertaling opent naast de selectie.

## Hoe het werkt

1. De client stuurt de tekst naar de server — met de bestandsnaam erbij wanneer de tekst in de editor of een diff was geselecteerd, zodat het model weet welke commentsyntax het bestand gebruikt;
2. De vertaal-LLM van de server vertaalt naar de doeltaal die in de Web UI is geconfigureerd. Markdown-opmaak (koppen, lijsten, tabellen, codeblokken) blijft behouden; bij bestandsselecties worden commentmarkeringen zoals `//` of `#` als decoratie herkend en verwijderd, terwijl code en identifiers onaangeroerd blijven;
3. De client rendert het resultaat als Markdown daar waar het verzoek vandaan kwam — ter plekke voor chatberichten, of in een scrollbare kaart voor editor- en diff-selecties.

## Wat je nodig hebt

- Een server met de vertaalfunctie ingeschakeld — de eenmalige configuratie staat in *Serverinstellingen* (sectie `LLM` van de Web UI);
- Een desktopclient die met die server is verbonden. Clientzijds is er niets te configureren: alle onderdelen pakken automatisch dezelfde serverinstellingen op.

Is vertaling niet geconfigureerd, dan toont de client de fout van de server (bijvoorbeeld in de vertaalkaart) in plaats van een resultaat.
