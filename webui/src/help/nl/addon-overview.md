Add-ons breiden de server uit met nieuwe mogelijkheden zonder de kern aan te raken: LLM-providers, spraakherkenningsbackends, dynamische-DNS-updaters, extra MCP-gereedschapsoppervlakken en de detectie van agent-apps bestaan allemaal als add-ons. De sectie `Addons` van de Web UI beheert ze.

## Categorieën

Elke add-on declareert een type, en de sectie groepeert de kaarten dienovereenkomstig:

- **Applications** — leren de terminal-observator een Code Agent CLI herkennen (Claude Code, Codex, Kimi Code, Pi en OpenCode zijn ingebouwd): procesdetectie, bezet/inactief-status, automatiseringshaken;
- **LLM Providers** — chat-completionbackends waaruit de AI-functies kiezen (een OpenAI-compatibele provider is ingebouwd);
- **Voice / ASR** — spraakherkenningsbackends (de ingebouwde `vibe-voice`-add-on wijst naar de begeleide lokale ASR-server);
- **Dynamic DNS** — houden een hostname op het huidige adres (Cloudflare is ingebouwd);
- **MCP** — extra MCP-gereedschapsoppervlakken op de MCP-host (EasyEDA Pro is ingebouwd);
- al het andere belandt in de groep **Other**.

## Ingebouwde en geüploade add-ons

- **Ingebouwde** add-ons zitten in het programma en zijn altijd beschikbaar. Hun manifest en bron blijven ongerept — een overschreven bron kan altijd worden hersteld;
- **Geüploade** add-ons zijn van uzelf: een zip met een manifest en een JavaScript-bron, geïnstalleerd via de knop `Upload Addon`;
- Het `Modified`-badge markeert een add-on waarvan u de bron hebt overschreven.

Elke kaart toont het type, de badges en een activatieschakelaar; uitgeklapt verschijnen beschrijving, auteur, configuratievelden en broncode.

![De sectie Addons — gegroepeerde kaarten met badges en activatieschakelaars](images/addon-section.jpg)

## Configuratie

Het manifest van een add-on declareert zijn configuratievelden, en de uitgeklapte kaart maakt er een formulier van. Waarden worden per add-on bewaard en werken alleen zolang hij actief is; wachtwoordvelden blijven gemaskeerd. Uitschakelen bewaart de configuratie — bij opnieuw activeren gaat u verder waar u was.
