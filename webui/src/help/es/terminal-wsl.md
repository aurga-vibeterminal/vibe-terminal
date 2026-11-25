Hay dos formas de usar WSL en Windows: la ligera añade un shell WSL como perfil de terminal, la completa instala vterm-server dentro de WSL. La primera solo te da un shell; la segunda desbloquea la gestión de archivos, el panel Git, MCP y todo lo demás.

## La forma ligera: un perfil de terminal WSL

En Windows, **los perfiles de terminal WSL se cargan automáticamente**: al arrancar, el servidor detecta `wsl.exe` y enumera las distribuciones instaladas, añadiendo una entrada de perfil por distribución a la lista (p. ej. `Ubuntu (WSL)`; aparece una entrada genérica `WSL` cuando no hay distribuciones listadas).

Para usar una, simplemente elige la entrada WSL en el menú de perfiles de la barra de título del cliente — la distribución arranca y te deja directamente en su terminal, igual que abrir una local.

Limitación: esas sesiones se ejecutan en el servidor del lado Windows — **el explorador de archivos, el panel Git, las variables de entorno y compañía del cliente no funcionan** (operan con semántica del sistema de archivos Windows y no ven el sistema de archivos Linux dentro de WSL).

![El menú de perfiles de terminal — las distribuciones WSL listadas automáticamente](images/terminal-wsl-profile.jpg)

## La forma completa: instalar vterm-server dentro de WSL

Trata WSL como su propia máquina Linux e instala el vterm-server headless. El sitio web ofrece un script de una línea — pégalo en un terminal WSL:

```bash
curl -fsSL https://vterminal.dev/install.sh | bash
```

El script automáticamente:

1. Detecta el entorno WSL/headless y descarga el vterm-server **headless** en `~/.local/vterm`;
2. Instala un **servicio de usuario systemd** y lo inicia de inmediato — se ejecuta en cada inicio de sesión (incluido el arranque de WSL), sin inicio manual;
3. Añade `~/.local/vterm` a tu PATH.

> Cuando WSL tiene systemd deshabilitado, el script te indica añadir `[boot]` `systemd=true` a `/etc/wsl.conf`, ejecutar `wsl --shutdown` en el lado Windows y volver a ejecutar el instalador.

## Abrir la página de configuración

Dentro de WSL:

```bash
vterm-server --open
```

El comando imprime la dirección de la interfaz web (`http://localhost:<config-port>/`) e intenta abrir un navegador (en WSL vía `wslview`, el navegador predeterminado de Windows). Gracias al reenvío de localhost de WSL2, **abrir la misma dirección en un navegador de Windows** funciona igual de bien. Mientras el servidor esté en marcha, la URL impresa lleva un token de acceso de un solo uso (`?ticket=...`, emitido por un socket local), así que la página se abre ya autenticada.

## (Opcional) Compartir el servidor WSL en el relay anfitrión

Si la máquina Windows también ejecuta vterm-server (actuando como relay), el servidor WSL puede acoplarse bajo él para que el cliente vea ambas «máquinas» a través de una conexión local:

1. En la interfaz web del servidor Windows, pulsa el código QR para copiar la cadena de uso compartido;
2. Abre la interfaz web del servidor WSL (dirección de la sección anterior), pulsa el botón de pegar que aparece junto al campo `Relay Address` para autorrellenar dirección y clave, luego `Apply`;
3. Mientras el vterm-server de Windows se ejecuta, el cliente ve los terminales, archivos y Git del servidor WSL bajo el relay local.

> Bajo el networking NAT por defecto de WSL2, WSL alcanza al anfitrión Windows mediante la dirección del anfitrión vista desde WSL (la puerta de enlace predeterminada — el nameserver en `cat /etc/resolv.conf`); con networking en espejo funciona `localhost`. Consulta el capítulo Relay para los detalles del relay.
