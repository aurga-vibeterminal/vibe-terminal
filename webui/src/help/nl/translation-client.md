Zodra de server is geconfigureerd, biedt elk onderdeel van de client vertaling aan via hetzelfde globe-icoon. Resultaten worden per item gecachet, dus heen en weer wisselen herhaalt nooit een verzoek.

## Berichten in het chatpaneel

Elke berichtkop heeft een vertaalicoon — naast de tijd bij je eigen berichten, rechts van de afzendernaam en tijd bij antwoorden van het model, en rechts van het label `Denkt na...` bij redeneerblokken:

- **Eerste klik** stuurt de rauwe Markdown van het bericht naar de server en wisselt de berichttekst om met de vertaling. Het icoon licht op zolang de vertaling wordt getoond;
- **Nog een klik** herstelt de originele tekst;
- De vertaling blijft aan het bericht kleven zolang het gesprek open blijft, ook aan `Denkt na...`-blokken zodra je ze uitklapt.

Vertaalde berichten renderen als gewone Markdown, dus lijsten, tabellen en codeblokken behouden hun vorm.

![Een chatbericht met het vertaalicoon in de kop en de vertaalde tekst eronder](images/translation-chat-message.jpg)

## Commitbericht-concept

In het commitvak van het Git-paneel verschijnt naast de AI-knop een vertaalknop zodra je concept meer dan één regel beslaat:

- **Klik** om het hele concept te vertalen — de vertaalde tekst vervangt het concept in het vak als alleen-lezen voorvertoning;
- **Klik opnieuw** om terug te keren naar je originele formulering (het concept zelf is nooit gewijzigd);
- Typen is uitgeschakeld zolang de voorvertoning wordt getoond; wissel eerst terug en bewerk daarna verder.

![Het commitvak met de vertaalknop en de vertaalde voorvertoning](images/translation-commit-draft.jpg)

## Commitdetail-popup

Hang met de muis boven een commit in de Git-geschiedenis om de detail-popup te openen. Naast de hash-kopieerknop zit een vertaalicoon: klik om het commitbericht in de popup te vertalen; klik opnieuw om het origineel te zien. Het vertaalde bericht rendert zoals het origineel — onderwerpregel, Markdown-tekst en codeblokken inbegrepen.

![De commitdetail-popup — kopieer- en vertaalknoppen op de hashregel](images/translation-git-tip.jpg)

## Selecties in de code-editor

Selecteer tekst in de code-editor — gewone en blok (rechthoekige) selecties werken allebei — klik met de rechtermuisknop en kies `Vertalen` (het eerste menu-item):

1. Een gevuld globe-badge verschijnt boven het regelnummer van de eerste geselecteerde regel;
2. Een vertaalkaart opent direct met een *Translating…*-placeholder en vult zich wanneer de server antwoordt;
3. De kaart is kleverig: hij blijft op zijn plek terwijl je scrolt, en alleen zijn sluitknop, `Esc` of een klik erbuiten zetten hem weg. Over het badge hangen brengt hem terug.

Elke vertaalde selectie krijgt een eigen badge, en dezelfde regels opnieuw vertalen toont gewoon de gecachete kaart weer. Badges verdwijnen wanneer je een ander bestand opent.

![Een selectie in de code-editor met Vertalen in het contextmenu](images/translation-editor-menu.jpg)

## Selecties in de diff-weergave

De diff-weergave werkt hetzelfde: selecteer regels in de diff-tekst, rechtermuisknop, `Vertalen`. Het badge vervangt het nieuwe regelnummer van de eerste geselecteerde regel, en de resultaatkaart ankert eronder. Selecties in de bewerkmodus vertalen ook; wisselen naar een ander bestand of commit wist de badges.

![Een diff met een vertaalde selectie — badge in de goot en vertaalkaart](images/translation-diff-card.jpg)

## Wat er wordt vertaald

De server vertelt het model waar de tekst vandaan komt:

- **Proza** (chatberichten, commitberichten) — de Markdown-opmaak blijft behouden en alleen de natuurlijke taal wordt vertaald;
- **Bestandsselecties** (editor, diff) — de bestandsnaam reist mee met het verzoek, dus het model kent de commentmarkeringen van het bestand (`//`, `#`, `--`, …). Commentmarkeringen worden verwijderd en de commenttekst wordt als proza vertaald; Markdown binnen het commentaar (koppen, lijsten, codeblokken) overleeft. Is de selectie gewone code, dan worden alleen comments en stringliteralen vertaald — identifiers en structuur blijven onaangeroerd.
