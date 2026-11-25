La interfaz web es la página de configuración propia del servidor: cada función del servidor se organiza en una sección, listada en la barra lateral izquierda. No es un terminal — el trabajo del día a día ocurre en el cliente de escritorio o en `vtermc` (véase Descripción general). Esta página explica la organización y el comportamiento común; los capítulos de funciones cubren los ajustes de cada sección en detalle.

## Distribución

- **Barra lateral** — la lista de secciones (abajo), el icono de ayuda junto al nombre del servidor y la versión de la aplicación en la parte inferior; en pantallas estrechas se desliza desde la izquierda y el botón de menú del encabezado la alterna;
- **Encabezado** — el título de la sección actual más, arriba a la derecha, el selector de idioma y el interruptor de tema claro/oscuro (en pantallas estrechas el icono de ayuda también está ahí);
- **Contenido** — el formulario de la sección seleccionada, centrado y desplazable.

## Secciones

En el orden de la barra lateral:

- `Servidor` — puerto, certificado TLS y ajustes de autenticación: la `Secret Key`, el `QR Code` de emparejamiento, la dirección del relay y el interruptor de inicio automático están aquí (véase Conectar un cliente y Guía de configuración). En Linux la pista de inicio automático también explica el inicio de sesión automático del gestor de pantalla para llegar al escritorio antes de un inicio de sesión local (véase Configuración del servidor en Escritorio remoto y simuladores);
- `Terminal` — análisis del terminal: detección de avisos, confirmación automática y clasificación por LLM (véase Confirmación automática);
- `UART` — perfiles de terminal serie respaldados por puertos serie locales (véase Puerto serie UART);
- `Complementos` — complementos instalados: actívalos, edítalos o súbelos (los proveedores LLM son complementos — véase LLM);
- `LLM` — asigna un proveedor LLM a cada función de IA (véase LLM);
- `Voz` — proveedor de reconocimiento de voz, modelo y ajustes de transcripción (véase Configuración del servidor en Entrada de voz);
- `Entorno` — colecciones de variables de entorno en un árbol de grupos (véase Variables de entorno);
- `MCP` — el servidor MCP de terminal expuesto a los agentes de código (véase Terminal MCP);
- `DDNS` — proveedor de DDNS y ajustes de sincronización (véase DDNS);
- `Redirección de puertos` — el grupo de puertos redirigidos y las redirecciones TLS activas (véase Redirección de puertos);
- `Static HTTP` — carpetas servidas por HTTP y las comparticiones de archivos activas (véase Publicar con servidor HTTP);
- `Escritorio remoto` — ajustes de captura, entrada y transmisión del escritorio remoto (véase Configuración del servidor en Escritorio remoto y simuladores);
- `Estadísticas de LLM` — uso de tokens, recuento de solicitudes y métricas de rendimiento de todas las funciones de LLM.

## Guardado y consejos

- La mayoría de las secciones se guardan automáticamente poco después de una edición — un indicador de guardado y cualquier error aparecen en la parte inferior de la sección;
- `MCP`, `DDNS` y `Escritorio remoto` retienen los cambios hasta que se pulsa `Guardar cambios`: mientras hay cambios pendientes aparece un aviso de `Cambios sin guardar` junto a los botones, y `Restablecer` los descarta;
- Abierta desde el cliente móvil, la interfaz web sigue el idioma y el tema de la aplicación y oculta sus propios controles de idioma y tema.
