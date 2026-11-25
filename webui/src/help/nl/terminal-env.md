Omgevingsvariabelen laten je gegroepeerde variabeleverzamelingen per project onderhouden (`ANDROID_HOME`, API-sleutels, proxy-instellingen) en ze met één klik op elke terminalsessie toepassen of wissen. Ze leven in de serverconfiguratie; waarden worden in rust versleuteld.

## Onderhouden in de Web UI

De sectie `Environment Variables` organiseert ze als een boom:

- **Groepen** — pure containers, nestbaar (rechtsklik → `New Group`), voor organisatie per project of scenario;
- **Collecties** — de bladeren van de boom en de werkelijke variabelehouders (`New Leaf`); alleen collecties kunnen op een terminal worden toegepast;
- De sleutel/waarde-paren van elke collectie worden bewerkt via `Edit Variables` (`Add Variable`); waarden zijn standaard verborgen;
- Collecties kunnen worden gedupliceerd om varianten af te leiden (dezelfde verzameling met één gewijzigde sleutel).

![De pagina Environment Variables — de groepen/collectieboom](images/terminal-env-tree.jpg)

## Toepassen op een terminal

Rechtsklik in een terminal → `Envs`:

- Het submenu spiegel de boom uit de Web UI; vouw een collectie uit en kies `Apply` om de variabelen in de huidige sessie te injecteren, of `Clear` om ze weer te verwijderen;
- Nieuw gestarte processen zien de variabelen (al draaiende voorgrondprogramma's niet);
- Collecties verschijnen opgevouwen onder hun groepen; het hele item `Envs` verbergt zich zolang de boom geen collecties heeft.

![Terminal-rechtsklik — toepassen en wissen in het Envs-submenu](images/terminal-env-menu.mp4)
