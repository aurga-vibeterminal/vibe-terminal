El cliente de escritorio abre las pantallas remotas desde la barra SCREENS en la parte superior del panel Debug. Cada pantalla vive en su propia ventana flotante utilizable junto a los terminales.

## Abrir la barra SCREENS

1. Conéctate al servidor destino (directamente o a través de un relay);
2. pulsa la pestaña `Debug` en la barra de actividad izquierda (el icono de herramientas);
3. la barra `SCREENS` aparece en la parte superior del panel: un botón de pantalla por cada monitor de escritorio físico del servidor, seguido de los conmutadores `iOS Simulator` y `Android Simulator`.

![El panel Debug — la barra SCREENS con los botones de pantalla y los conmutadores de simulador](images/remote-desktop-client-screens.jpg)

## Ventanas de pantalla de escritorio

Pulsa un botón de pantalla para abrir la ventana de ese monitor:

- la ventana se redimensiona libremente (hasta 800×600) y la imagen se reorganiza con ella;
- los servidores multimonitor reciben una ventana por pantalla; varias pueden estar abiertas a la vez;
- mientras conecta, la ventana muestra un marcador «preparando escritorio»; si al servidor le falta permiso de captura, la ventana muestra directamente las instrucciones de permiso de la plataforma;
- vuelve a pulsar el botón de la barra o cierra la ventana para dejar de mirar (el estado activo del botón se mantiene sincronizado con la ventana).

## Ventana del simulador iOS

Pulsa `iOS Simulator` para abrir una ventana con marco de teléfono de tamaño fijo (264×640 vertical):

- el selector de dispositivo de arriba cambia el aspecto del teléfono; la ventana se reajusta a la resolución elegida;
- `Rotate` alterna vertical/horizontal (ventana 524×380 en horizontal);
- requiere un servidor macOS con el simulador iOS de Xcode en ejecución.

## Ventana del emulador Android

`Android Simulator` funciona igual: marco de teléfono fijo, selector de dispositivo, botón de rotación. Las fuentes transmisibles incluyen emuladores AVD en ejecución y dispositivos reales conectados; un AVD también puede arrancarse o apagarse bajo demanda desde el cliente (véase Dispositivos Android).

> Las ventanas que transmiten un servidor localhost se excluyen automáticamente de la captura de pantalla para evitar un espejo de realimentación; las ventanas que transmiten una máquina remota permanecen capturables para que una grabación exterior siga mostrándolas.
