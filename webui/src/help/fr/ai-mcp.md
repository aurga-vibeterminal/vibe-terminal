Le serveur embarque un hôte Model Context Protocol intégré, et la section `MCP` de la Web UI le configure : les Code Agents (Claude Code, Codex, Kimi Code, Pi, OpenCode, Qwen Code, DeepSeek Harness, …) peuvent se connecter depuis leur propre chat et travailler directement sur votre machine — lire et piloter des terminaux et même piloter EasyEDA Pro.

## Réglages du serveur

Activez le serveur MCP et configurez :

- `Port` (`6180` par défaut) et `Bind Address` — `127.0.0.1` limite l'endpoint au localhost ; `0.0.0.0` l'ouvre au réseau ;
- `Access Token` — requis pour les requêtes hors localhost : les appelants distants doivent le présenter comme Bearer token, tandis que les requêtes venant du localhost sont toujours admises. Il n'est jamais vide — une installation neuve embarque un jeton aléatoire ; le dé le régénère et le bouton copie le remet ;
- `Stream Throttle` — l'intervalle minimal entre les poussées de flux d'écran vers les agents connectés.

Les changements s'appliquent au `Save` ; le serveur se recharge à chaud sans redémarrage.

## Endpoints

Chaque ensemble de capacités est servi sur sa propre URL, `http://localhost:{port}/{id}/mcp` :

- **Terminal MCP** (`/terminal/mcp`) — la surface terminal intégrée. Les agents peuvent lister les terminaux, lire l'écran visible et l'historique, saisir texte brut et touches nommées, redimensionner et attendre l'invite. Les appels ciblent un terminal par son Terminal ID — clic droit sur un terminal → `MCP` → `Copy Terminal ID`, puis collez l'ID dans le contexte de l'agent (voir Terminal MCP dans la section Terminals) ;
- **EasyEDA Pro MCP** (`/easyeda/mcp`) — un addon MCP intégré qui ponte EasyEDA Pro vers les agents : un petit jeu d'outils fixe plus un catalogue interrogeable d'appels `eda.*`, transmis à l'extension connecteur EasyEDA via WebSocket. Activez d'abord l'addon dans la section `Addons` — son endpoint apparaît dans la liste une fois installé, et le badge indique si le démon de l'addon est connecté. Le champ `Timeout` de l'addon borne chaque requête pontée (augmentez-le pour les opérations EDA lentes) ;
- **Autres addons MCP** — tout addon de type MCP ajoute son endpoint de la même façon, chacun avec un badge connecté/déconnecté en direct.

Choisissez votre CLI d'agent dans la liste déroulante au-dessus ; le bouton copie à côté d'un endpoint vous donne la commande de configuration prête à exécuter.

## Connecter un agent

Le jeton ne se colle jamais à la main. Le serveur maintient un profil d'environnement `Vibe Terminal MCP Bearer Token` généré automatiquement (voir la section Environment Variables) qui exporte `VTERM_MCP_TOKEN` ; appliquez-le au terminal avant de connecter l'agent, puis utilisez la commande copiée :

- **Apps à commande shell** (Claude Code, Codex) — exécutez la commande `mcp add` copiée dans un terminal ordinaire. Codex lit le jeton depuis `VTERM_MCP_TOKEN` via `--bearer-token-env-var`, aucun secret ne finit dans la ligne de commande ;
- **Apps conversationnelles** (Kimi Code, Pi, OpenCode, Qwen Code, DeepSeek Harness) — démarrez l'app avec le profil appliqué, puis envoyez l'invite copiée dans sa conversation. L'agent met directement à jour sa configuration MCP. Kimi utilise `bearerTokenEnvVar` ; Pi, OpenCode et Qwen Code utilisent le jeton Bearer de `VTERM_MCP_TOKEN`, qui n'apparaît donc jamais dans le chat.

Dans Pi et Qwen Code, `/mcp` inspecte les connexions.

La carte `Command Templates` sous les réglages conserve les commandes par application : modifiez une entrée, surchargez une intégrée ou ajoutez des gabarits pour de nouvelles CLI — l'aperçu montre exactement ce que le bouton copie produira.
