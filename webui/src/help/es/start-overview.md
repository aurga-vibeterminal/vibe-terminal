Vibe Terminal es un hub de terminales gestionado de forma remota para el desarrollo embebido y de servidores. Un **servidor** se ejecuta en cada máquina con la que trabajas — un equipo de desarrollo, un servidor de compilación, una placa embebida — y los **clientes** se conectan a él desde una aplicación de escritorio o la línea de comandos. Esta guía cubre ambas partes: la interfaz web que configura un servidor y los clientes que trabajan con él.

## Cómo encaja todo

- Todo se ejecuta en el **servidor** — las sesiones de terminal viven allí, así que cerrar un cliente, perder la red o reorganizar las ventanas nunca interrumpe un comando en ejecución; reconecta y continúa donde lo dejaste;
- Los clientes abren una única conexión WebSocket cifrada (`wss://host:port`; `wss://localhost:5153` para el servidor local) y se autentican con la **clave secreta** del servidor; el emparejamiento entrega la clave mediante un código QR o una cadena de uso compartido cifrada (véase Conectar un cliente);
- La **interfaz web** es la página de configuración del propio servidor — perfiles, addons, proveedores LLM, relay, redirección de puertos y más. No es un terminal: el trabajo del día a día ocurre en el cliente de escritorio o en `vtermc`.

## Componentes

- **Servidor** — se ejecuta en la máquina de destino; aloja las sesiones de terminal y sirve archivos, Git, escritorio remoto, voz y funciones de IA a los clientes conectados;
- **Cliente de escritorio** — el espacio de trabajo nativo: terminales con divisiones, grupos de pestañas y ventanas desprendidas, un explorador de archivos y un editor, herramientas de Git, entrada de voz, historial de conversación del agente de código, escritorio remoto y depuración remota;
- **CLI `vtermc`** — lanzar y adjuntarse a sesiones desde cualquier terminal sencillo, por SSH o en una máquina headless; comparte su lista de servidores con el cliente de escritorio (véase Cliente de línea de comandos);
- **Interfaz web** — ajustes del propio servidor en el navegador, organizados en secciones en la barra lateral izquierda (véase Uso de la interfaz web).

## Icono de bandeja

En las máquinas con una sesión de escritorio, el servidor muestra un icono en la bandeja mientras se ejecuta:

![Menú del icono de bandeja — Open pone en primer plano un cliente en ejecución o lo lanza, Settings abre la interfaz web y la entrada de voz inicia o detiene el servidor de voz](images/start-tray-menu.jpg)

- **Open** — pone en primer plano un cliente de escritorio en ejecución, o lo lanza cuando no hay ninguno en ejecución (hacer doble clic en el icono hace lo mismo);
- **Settings** — abre la interfaz web del servidor en el navegador (véase Uso de la interfaz web);
- **Start/Stop Vibe Voice** — inicia o detiene el servidor de voz sin abrir la interfaz web, replicando los botones de iniciar/detener de la sección `Voice`; aparece cuando el reconocimiento de voz está activado, y la etiqueta refleja el estado (p. ej. `Stop Vibe Voice (model on device)` mientras el modelo está cargado);
- **Exit** — apaga el servidor.

Mientras las carpetas se publican por HTTP, una entrada `HTTP Servers` permite detener cada uso compartido con un solo clic (véase Publicar con servidor HTTP).

## Mapa de funciones

El resto de esta guía sigue los capítulos de la barra lateral:

- **Terminales** — crear, dividir y organizar terminales; perfiles, WSL y puerto serie UART; comandos rápidos y variables de entorno; controlar terminales desde agentes externos con MCP; revisar las conversaciones del agente de código; confirmar automáticamente los avisos interactivos;
- **Archivos** — proyectos recientes, el explorador de archivos del lado servidor, subir y descargar, y publicar un directorio por HTTP (véase Explorador de archivos, Subir y descargar y Publicar con servidor HTTP);
- **Git** — staging, commits asistidos por IA, worktrees, rebase interactivo, resolución de conflictos y push;
- **Escritorio remoto y simuladores** — ver y controlar las pantallas del servidor, dispositivos Android y emuladores, y el simulador de iOS;
- **Entrada de voz** — reconocimiento de voz del lado servidor con dictado push-to-talk y pulido LLM;
- **Relay** — alcanzar máquinas detrás de NAT o con IPs dinámicas a través de un único punto de entrada público;
- **Acceso remoto** — dominios DDNS y redirección de puertos TLS para servicios que se ejecutan en el servidor;
- **Funciones de IA** — proveedores LLM como addons, asignados por función, más la integración con MCP (véase LLM y MCP).

## Por dónde empezar

1. Abre la sección `Server` de la interfaz web, establece una `Secret Key` y empareja un cliente — Conectar un cliente muestra la configuración con código QR y la manual;
2. Recorre las secciones de la interfaz web para configurar perfiles, addons y funciones — véase Uso de la interfaz web;
3. Abre un terminal en el servidor y recorre los capítulos anteriores según necesites.
