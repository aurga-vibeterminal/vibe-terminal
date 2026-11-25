Druk in de desktopclient overal snel twee keer op `Ctrl` (binnen ~300 ms) om het spraakinvoervenster te openen.

## Opnemen

Het venster start direct met opnemen: de vloeibare golfvorm volgt je stem, de bovenkiezer wisselt microfoons (de keuze wordt onthouden), en een timer toont de verstreken tijd. `Stop` beëindigt de clip en stuurt hem naar transcriptie (`Reset` start opnieuw).

![De opnamefase — golfvorm, microfoonkiezer en timer](images/voice-client-record.jpg)

## Verwerken

Terwijl de server de audio transcript speelt een deeltjesstroom-animatie.

## Bewerken

De transcriptie belandt in een bewerkbaar tekstvak:

- De moduspil wisselt tussen `original` en `polished`; `Polish` draait de LLM eenmaal, en `Auto polish` doet dat direct na elke transcriptie;
- `Re-record` gooit de tekst weg en begint over; `Insert` neemt opnieuw op en vlecht de nieuwe transcriptie op de caret in de bestaande tekst (eventuele selectie vervangend), zodat je wat je al dicteerde kunt uitbreiden of corrigeren — de caret belandt direct na het ingevoegde fragment;
- Zweef over het tekstvak om rechtsboven een kopieerknop te onthullen: één klik kopieert de tekst en hij verandert in een vinkje; tekst bewerken of modus wisselen maakt hem weer gereed voor een nieuwe kopie;
- `Submit` (of `Ctrl+Enter`) typt de tekst de actieve terminal in; `Esc` sluit het venster.

![De bewerkfase — moduspil, auto-bijschaving, de actierij met Insert, en de kopieerknop bij zweven](images/voice-client-editing.jpg)

## Bijschaving instellen

`Polish` heeft een spraak-bijschavings-LLM nodig: schakel in de sectie `LLM` van de Web UI `Enable voice text polish` in de per-functie-lijst in (een gewone provider plus deze schakelaar is voldoende). Zonder die blijft de knop `Polish` grijs en is de optie automatisch bijschaven verborgen.
