Todo vive en la sección `Remote Desktop` de la interfaz web; la página muestra los ajustes que aplican a la plataforma del servidor.

## Interruptor principal

`Enable Remote Desktop` — una vez activado, los clientes remotos conectados pueden ver el escritorio de esta máquina y enviar entrada.

## Ajustes por plataforma

**Windows**

- `Windows Service` — registra el servidor de pantalla como servicio del sistema de autoarranque para que el escritorio remoto sobreviva a reinicios sin sesión iniciada. `Install` / `Uninstall` dispara un aviso UAC en el servidor (el botón muestra progreso mientras espera); la insignia de estado distingue en ejecución / instalado / no instalado.
- `Máquinas headless (sin monitor)` — sin una pantalla conectada (o despierta), Windows no mantiene ningún escritorio capturable. La página enlaza la descarga del controlador de código abierto Virtual Display Driver (ARM64 / x64 elegido automáticamente); instálalo para que siempre exista un escritorio virtual.

![Interfaz web — la sección Remote Desktop en Windows (tarjetas de servicio y headless)](images/remote-desktop-server-windows.jpg)

**Linux**

- `Unlock on Connect` — desbloquea automáticamente la sesión bloqueada al iniciar una transmisión (donde el entorno lo soporta);
- la captura de pantalla pasa por el Portal; el `Reset Permission` de la tarjeta `Screen Recording Permission` borra un consentimiento guardado del Portal para que puedas elegir otra pantalla;
- `Auto-start on Login` (en la sección `Servidor`) — en hosts con gestor de pantalla el servidor arranca junto con la sesión gráfica, de modo que la captura hereda el entorno de escritorio completo; en hosts headless arranca durante el arranque, antes de cualquier inicio de sesión. Para llegar al escritorio cuando nadie ha iniciado sesión localmente, activa también el inicio de sesión automático en el gestor de pantalla (GDM, SDDM, …); por seguridad el bloqueo automático de pantalla puede permanecer activado — la sesión y el servidor siguen en marcha con la pantalla bloqueada, y `Unlock on Connect` (o `loginctl unlock-session`) abre la sesión de forma remota.
![Interfaz web — la tarjeta Screen Recording Permission (Linux)](images/remote-desktop-server-permission.jpg)


**macOS**

- `Screen Recording Permission` — el sistema exige aprobación explícita para la captura. Pulsa `Grant Permission` para abrir el diálogo del sistema; `Reset Permission` se recupera de un estado atascado.

## Herramientas de transmisión de pantalla (soporte Android)

`Android SDK directory` y `Android Studio directory` localizan adb y el emulador; déjalas vacías para autodetección. Una vez establecidas, la página muestra la ruta resuelta y su procedencia (auto / env / override); `Reset to Auto-Detect` restaura el valor por defecto. Véase Dispositivos Android para lo que habilitan.

## Codificación de la transmisión

- `Encoder` — Auto o un backend concreto; el servidor lista los codificadores utilizables y atenúa los no disponibles con el motivo;
- `Frame rate` — 60 / 30 / 15 fps;
- `Max resolution` — techo del lado de captura (4K / 2.5K / 2K);
- `Bitrate` — `Auto` escala por área con la resolución (tasa base hasta 1080p, ~4× en 4K, con tope de 40 Mbps); un valor fijo se aplica de inmediato a las transmisiones en curso, mientras que codificador / fps / resolución se aplican a transmisiones iniciadas después;
- el pie de la tarjeta muestra el codificador / fps / resolución / tasa de bits vigentes.

![Interfaz web — ajustes de codificación de transmisión](images/remote-desktop-server-stream.jpg)

Pulsa `Save` para aplicar los cambios.
