Además de las pantallas de escritorio, el escritorio remoto puede transmitir dispositivos Android — teléfonos reales por adb y emuladores AVD. El cliente los muestra en una ventana con marco de teléfono con entrada táctil, rotación y control remoto del ciclo de vida del emulador.

## Requisitos previos

El servidor necesita encontrar la cadena de herramientas de Android:

- **SDK de Android** (proporciona `adb`) — necesario para dispositivos y emuladores;
- **Android Studio** — necesario para arrancar emuladores AVD.

Ambos se autodetectan por defecto. Si la detección falla o las herramientas viven fuera de las ubicaciones estándar, ajusta `Android SDK directory` / `Android Studio directory` bajo `Remote Desktop` → `Screen streaming tools`; los cambios surten efecto de inmediato (aplicados en caliente, sin reiniciar el servidor).

## Fuentes de dispositivos

Con las rutas listas, el servidor enumera automáticamente las fuentes Android disponibles:

- **Dispositivos reales conectados** — conéctalos por USB con la depuración USB activada; todo lo que vea `adb devices` puede transmitirse;
- **Emuladores AVD** — los emuladores en ejecución aparecen solos; también puedes arrancar un AVD elegido bajo demanda desde el cliente (sin necesidad de arrancarlo a mano primero) y apagarlo remotamente.

Cada fuente recibe su propia entrada en la barra SCREENS del cliente.

## Orientación

El botón de rotación de la ventana del emulador ajusta el `user_rotation` del sistema a través de `adb` — cambia la dirección de pantalla del propio dispositivo, y la ventana se reajusta a la nueva resolución, adaptándose tanto a vertical como a horizontal.

## Lado del cliente

Abre la pestaña `Debug` en el panel izquierdo del cliente de escritorio y pulsa el botón `Android Simulator` de la barra SCREENS (solo aparece cuando el servidor expone fuentes Android) para abrir la ventana del emulador; el selector de dispositivo cambia el aspecto del marco y `Rotate` invierte la orientación. Véase Uso del cliente.

> La inyección táctil en dispositivos reales usa el canal de entrada de adb (Android 4.4+); algunas ROM personalizadas por el fabricante la restringen, en cuyo caso el dispositivo es de solo visualización.
