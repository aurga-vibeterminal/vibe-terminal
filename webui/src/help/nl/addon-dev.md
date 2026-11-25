Er zijn twee niveaus van aanpassing: de bron van een bestaande add-on patchen, of een nieuwe schrijven naar het voorbeeld van een ingebouwde en die uploaden.

## Hot-patchen van bron

De potloodknop op elke add-onkaart opent de JavaScript-bron om te bewerken. Opslaan slaat een momentopname op die het origineel overschrijft — voor ingebouwde én geüploade add-ons — en de wijziging werkt onmiddellijk, zonder herstart. Handig om een prompt bij te schaven, een URL te corrigeren of een idee live te proberen.

`Reset` gooit de momentopname weg en herstelt de oorspronkelijke bron (alleen ingebouwde — een geüploade add-on is al van u).

![De bron van een add-on bewerken — de opgeslagen momentopname overschrijft het origineel](images/addon-edit-source.jpg)

## Uw eigen add-on schrijven

Een add-on bestaat uit twee bestanden:

- `<id>.json` — het manifest: `id`, `name`, `description`, `author`, `email`, `date`, `type`, en `config_fields` (key, label, veldtype, standaardwaarde, beschrijving) waar de Web UI een formulier van maakt;
- `<id>.js` — de bron: een gewoon JavaScript-bestand (uitgevoerd door de ingebouwde QuickJS-engine, met een `addon.get_config()`-brug naar de geconfigureerde velden) dat de functies exporteert die zijn type aanroept — bijv. `chat_completion()` voor een LLM-provider, toollijsten en brugaanroepen voor een MCP-add-on.

De snelste start is een bestaande add-on exporteren: de knop `Export` op zijn kaart downloadt een zip met manifest en bron samen. Kies de ingebouwde add-on die het dichtst bij uw doel ligt (ze staan ook in `crates/vterm-server/src/addon/builtins/`), pas id, name, type en de geëxporteerde functies aan, en installeer het resultaat met `Upload Addon`. Geüploade add-ons kunt u weer verwijderen met de prullenbakknop.

![Een add-on exporteren — manifest en JavaScript-bron als één zip](images/addon-export.jpg)

## Opslag

Geüploade add-ons en bronmomentopnamen staan als `<id>.json` + `<id>.js` in de map `addons/` van de configuratiemap en overleven herstarts. Voor een ingebouwde id wordt alleen de `.js`-overschrijving gehonoreerd — de ingebouwde add-on houdt zijn eigen manifest.
