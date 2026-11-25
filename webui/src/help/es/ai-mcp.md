El servidor incluye un host de Model Context Protocol integrado, y la sección `MCP` de la Web UI lo configura: los Code Agents (Claude Code, Codex, Kimi Code, Pi, OpenCode, Qwen Code, DeepSeek Harness, …) pueden conectarse desde su propio chat y trabajar directamente en tu máquina — leer y controlar terminales e incluso operar EasyEDA Pro.

## Ajustes del servidor

Activa el servidor MCP y configura:

- `Port` (por defecto `6180`) y `Bind Address` — `127.0.0.1` mantiene el endpoint solo en localhost; `0.0.0.0` lo abre a la red;
- `Access Token` — obligatorio para peticiones que no vienen de localhost: los llamadores remotos deben presentarlo como Bearer token, mientras que las peticiones de localhost siempre se admiten. Nunca está vacío — una instalación nueva trae uno aleatorio; el botón de dado lo regenera y el de copia lo entrega;
- `Stream Throttle` — el intervalo mínimo entre emisiones de flujo de pantalla hacia los agentes conectados.

Los cambios se aplican al `Save`; el servidor se recarga sin reiniciar.

## Endpoints

Cada conjunto de capacidades se sirve en su propia URL, `http://localhost:{port}/{id}/mcp`:

- **Terminal MCP** (`/terminal/mcp`) — la superficie de terminal integrada. Los agentes pueden listar terminales, leer la pantalla visible y el historial, teclear entrada cruda y teclas con nombre, redimensionar y esperar el prompt. Las llamadas apuntan a una terminal por su Terminal ID — clic derecho en una terminal → `MCP` → `Copy Terminal ID` y pega el ID en el contexto del agente (véase Terminal MCP en la sección Terminals);
- **EasyEDA Pro MCP** (`/easyeda/mcp`) — un addon MCP integrado que tiende un puente entre EasyEDA Pro y los agentes: un pequeño conjunto fijo de herramientas más un catálogo buscable de llamadas `eda.*`, reenviadas a la extensión conectora de EasyEDA por WebSocket. Activa primero el addon en la sección `Addons` — su endpoint aparece en la lista una vez instalado, y la insignia indica si el demonio del addon está conectado. El campo `Timeout` del addon limita cada petición puente (súbelo para operaciones EDA lentas);
- **Otros addons MCP** — cualquier addon de tipo MCP añade su endpoint igual, cada uno con insignia conectado/desconectado en vivo.

Elige tu CLI de agente en el desplegable sobre la lista; el botón de copia junto a un endpoint te entrega el comando de configuración listo para ejecutar.

## Conectar un agente

El token nunca se pega a mano. El servidor mantiene un perfil de entorno `Vibe Terminal MCP Bearer Token` generado automáticamente (véase la sección Environment Variables) que exporta `VTERM_MCP_TOKEN`; aplícalo a la terminal antes de conectar el agente y usa después el comando copiado:

- **Apps de comando de shell** (Claude Code, Codex) — ejecuta el comando `mcp add` copiado en una terminal normal. Codex lee el token desde `VTERM_MCP_TOKEN` vía `--bearer-token-env-var`, así ningún secreto acaba en la línea de comandos;
- **Apps de conversación** (Kimi Code, Pi, OpenCode, Qwen Code, DeepSeek Harness) — inicia la app con el perfil aplicado y envía el prompt copiado dentro de su conversación. El agente actualiza directamente su configuración MCP. Kimi usa `bearerTokenEnvVar`; Pi, OpenCode y Qwen Code usan el token Bearer de `VTERM_MCP_TOKEN`, por lo que el token nunca aparece en el chat.

Dentro de Pi y Qwen Code, `/mcp` inspecciona conexiones.

La tarjeta `Command Templates` bajo los ajustes guarda los comandos por aplicación: edita una entrada, sobrescribe una integrada o añade plantillas para nuevas CLIs — la vista previa muestra exactamente lo que producirá el botón de copia.
