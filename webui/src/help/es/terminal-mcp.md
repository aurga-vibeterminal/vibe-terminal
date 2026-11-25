Cada terminal es un ciudadano de primera clase de MCP (Model Context Protocol): los agentes externos conducen terminales mediante las herramientas MCP del servidor — listar terminales, leer la pantalla, ejecutar comandos, esperar a que terminen — permitiendo que el agente de código que ya usas opere shells interactivos reales.

## Las herramientas MCP de terminal del servidor

En torno a las sesiones de terminal, un cliente MCP puede:

- `list_terminals` — todos los terminales abiertos; `get_terminal` — título, cwd y estado de aplicación de un terminal;
- `get_screen` — el texto visible actual de la pantalla; `get_scrollback` — líneas recientes del historial;
- `send_command` — ejecutar una línea de comando (Enter añadido); `send_input` — texto bruto sin Enter (confirmar `y`, rellenar formularios); `send_key` — teclas con nombre (Enter, Ctrl+C, flechas);
- `clear_terminal` — vaciar el búfer; `resize_terminal` — redimensionar el PTY; `wait_terminal` — bloquear hasta que el terminal deje de estar ocupado.

## Direccionar un terminal: el Terminal ID

Las llamadas a herramientas se dirigen a un terminal por su Terminal ID. Clic derecho dentro de un terminal → `MCP` → `Copy Terminal ID`, luego pega el ID a tu agente para que conduzca exactamente ese terminal.

![Clic derecho del terminal — el submenú MCP](images/terminal-mcp-menu.jpg)

## Registro de aplicaciones de agente con un clic

El submenú `MCP` también lista las aplicaciones de agente detectadas en el servidor (CLI de editor y similares). Pulsar una aplicación teclea un comando `mcp add` listo en el terminal — **sin salto de línea final**; revísalo y pulsa tú mismo Enter para registrar el servicio MCP de este servidor con ese agente. Las entradas se ocultan mientras una aplicación ya se ejecuta en la sesión, para que el texto no caiga en su interfaz.

> Bucle típico: copiar el Terminal ID a tu agente → `send_command` inicia una compilación → `wait_terminal` hasta terminar → `get_screen` lee el resultado. Junto con la Confirmación automática (véase el capítulo de terminales), las tareas interactivas largas se ejecutan casi sin supervisión.
