La entrada de voz convierte el habla en entrada de terminal: habla en lugar de teclear, con pulido LLM opcional de la transcripción antes de enviar. El reconocimiento corre como servidor ASR local con modelos en el dispositivo — el audio y el texto se quedan en tu máquina. Esta página cubre el lado del servidor; la experiencia de escritorio vive en Uso del cliente.

## La tarjeta Local ASR Server

Abre la sección `Voice` de la interfaz web y activa el reconocimiento de voz. La tarjeta `Local ASR Server` gestiona el proceso vterm-voice supervisado:

- `Load on Startup` — arrancar el servidor ASR junto con el backend;
- `Bind Address` (`127.0.0.1` solo localhost / `0.0.0.0` todas las interfaces) y `Port` (por defecto `8398`) — dónde escucha el punto de acceso ASR;
- `Model Directory` + `Local Model` — elige del catálogo integrado: familias Qwen3-ASR, Fun-ASR y Parakeet, cada una en variantes full / 8 bits / 4 bits. `Download` trae el modelo desde Hugging Face o ModelScope al directorio con una barra de progreso en vivo; los comandos `git clone` manuales se muestran como alternativa, y los modelos ya en disco se marcan disponibles;
- `Device` — el backend de inferencia (`Auto`, `CPU`, `CUDA`, `Vulkan`, `Metal`), tal como lo informa el servidor;
- `Access Token` — secreto compartido del punto de acceso; el botón de dado genera uno;
- `Save` persiste la tarjeta (las ediciones son un borrador local hasta entonces), y `Start Server` / `Stop Server` controlan el proceso. El icono de estado del título de la tarjeta refleja la accesibilidad y el estado del modelo (cargado / cargando / error).

![La sección Voice — la tarjeta Local ASR Server con modelo, dispositivo y token](images/voice-server-local-asr.jpg)

> **Contención CUDA**: cuando otro programa de inferencia CUDA ya ocupa la GPU (p. ej. llama.cpp corriendo un modelo local), dejar que vterm-voice use también `CUDA` puede ralentizar drásticamente a ambos. En ese caso pon `Device` en `CPU` — estos modelos ASR compactos son suficientemente rápidos en CPU.

## Proveedor y pruebas

El desplegable de proveedores lista addons ASR — el addon `vibe-voice` integrado apunta a este servidor supervisado, así que elegirlo es la forma de que el panel use tus modelos locales.

La zona `Test` lo prueba todo sin cliente: `Record` (micrófono → forma de onda → transcripción) y `Live` (reconocimiento en flujo), cada uno con tiempos y botón de copia.

![La zona Test — las pestañas Record y Live](images/voice-server-test.jpg)
