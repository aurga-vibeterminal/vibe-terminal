El serie UART convierte la consola serie de un dispositivo (puerto de depuración, placa embebida, SBC) en una pestaña de terminal que puedes observar y con la que interactuar como con cualquier otro terminal.

## Configurar un puerto

En la sección `UART` de la interfaz web → `Add UART Port`:

- `Name` — se muestra como título del terminal (p. ej. «Debug Console»);
- `Serial Port` — elige entre los puertos detectados (el recuento se muestra arriba en la página);
- `Baud Rate` / `Data Bits` / `Parity` / `Stop Bits` / `Flow Control` — deben coincidir con el firmware del dispositivo;
- `Terminal Type` y `Encoding` — la codificación es Auto (UTF-8) por defecto y puede seguir al firmware;
- `Show Advanced Options` expone más parámetros serie.

Los puertos guardados aparecen en la lista `Configured UART Ports` y pueden editarse o eliminarse en cualquier momento.

![La página de ajustes UART — el formulario de puerto](images/terminal-uart-config.jpg)

## Uso en el cliente

Los puertos configurados se abren como pestañas de terminal igual que las sesiones ordinarias del servidor:

- los datos serie se muestran a medida que llegan; la entrada del teclado va directa al puerto;
- el menú contextual ofrece `Clear` (vaciar la pantalla) y un submenú `Encoding` (cambiar codificación en vivo; la actual queda marcada);
- las sesiones UART no tienen funciones de shell (sin Comandos rápidos, Apps ni elementos de menú Envs).

Usos típicos: consolas de router/switch, registros de placas embebidas, supervisión de bancos de flasheo.
