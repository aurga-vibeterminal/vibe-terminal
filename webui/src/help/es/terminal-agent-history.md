Cuando ejecutas un agente de código (Claude Code, Codex, Kimi, Pi, OpenCode…) en un terminal, su conversación queda atrapada en esa ventana de terminal — desplazada entre salidas de comandos, perdida en cuanto la pantalla avanza. El `Historial de conversación del agente de código` eleva la conversación completa a un panel dedicado: cada intercambio mostrado estructurado, conversaciones pasadas conmutables, mensajes traducibles — y puedes enviar comandos al terminal directamente desde el panel.

## Abrir el panel

Pulsa el botón de chat (`AI`) en el lado derecho de la barra de título para alternar el panel Copilot. El desplegable de su parte superior lista **todas las conversaciones pasadas** del agente que se ejecuta en la sesión de terminal activa, la más reciente primero; elegir una la carga.

![El panel Copilot — el desplegable de conversaciones y el flujo de mensajes](images/terminal-agent-history.jpg)

## Cómo funciona

- El servidor sabe qué aplicación de agente se ejecuta en el terminal y analiza sus **archivos de sesión nativos** (el formato JSONL/historial de cada agente bajo su directorio de proyecto); los cambios del archivo llegan en vivo, manteniendo la vista de conversación al día;
- Agentes compatibles: `claude` / `codex` / `kimi` / `pi` / `opencode`; otras aplicaciones aún no tienen panel de conversación;
- Los mensajes llevan un rol (tú / agente) y un tipo (texto / razonamiento **Thinking**), junto con el nombre del modelo y la hora, renderizados como Markdown.

## Funciones del panel

- **Cambiar de conversación**: el desplegable `Conversations` de arriba se mueve entre sesiones pasadas, cada entrada muestra un resumen y su hora;
- **Traducir**: cualquier mensaje del agente puede traducirse a petición; la traducción reemplaza el texto en su sitio (y puede conmutarse de vuelta);
- **Enviar comandos directamente**: escribe en el cuadro inferior (Shift+Enter para salto de línea) y pulsa Enter — el comando va directo al terminal actual, como teclear en el propio cuadro de entrada del agente sin ocupar la vista del terminal.

## Encaje con otras funciones

- El panel y el terminal son dos vistas de la misma sesión: las preguntas del agente en el terminal se siguen respondiendo allí (o vía Confirmación automática); el panel sirve para revisar el historial, no para interceptar la interacción;
- Combinado con `Terminal MCP` (página anterior), los agentes externos también pueden leer la pantalla del terminal y ejecutar comandos — la conversación mostrada aquí más las acciones MCP forman el registro completo del trabajo del agente.
