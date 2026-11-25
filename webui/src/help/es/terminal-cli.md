`vtermc` es el cliente de línea de comandos: se conecta a un servidor, crea o se adjunta a sesiones de terminal y gestiona la lista de servidores guardada — todo dentro de un terminal sencillo, por SSH o en una máquina headless. Comparte `~/.vibeterminal/client.json` con el cliente de escritorio, así que ambos ven los mismos servidores.

## Iniciar un terminal

Ejecuta `vtermc` sin indicadores de modo para crear un terminal nuevo en el servidor local (`wss://localhost:5153` por defecto) con el shell detectado, y transmitirlo al terminal actual. Escribe `exit` (o `Ctrl+D`) en el shell remoto para finalizar la sesión.

Las sesiones viven en el servidor: si la conexión se corta — pérdida de red, cerrar la ventana, matar `vtermc` — la sesión sigue ejecutándose y puede readjuntarse luego con `vtermc -s`.

- `vtermc -p <id>` — iniciar con un perfil de terminal específico (la lista de perfiles está en la sección `Profiles` de la interfaz web);
- `vtermc -p` — sin id de perfil: el selector lista los perfiles del servidor interactivamente;
- `vtermc -w <dir>` — establecer el directorio de trabajo de la nueva sesión.

Cuando stdin no es un terminal (una tubería o archivo), `vtermc` cambia a modo por lotes: stdin se alimenta a la sesión y la salida fluye hasta que la sesión se cierra — útil desde scripts o agentes:

```sh
echo "make test" | vtermc
```

## Conectarse a un servidor

- `--server wss://host:port` — conectarse a un servidor concreto en lugar del local (direcciones IPv6 entre corchetes: `wss://[fd00::1]:5153`); la variable de entorno `VTERM_SERVER` establece el mismo valor por defecto;
- El apretón de manos se responde con la clave secreta del servidor: los servidores añadidos con `-a`/`-s` llevan su clave en `client.json`; el servidor local recurre a la variable de entorno `TERMINAL_GUI_SECRET_KEY` (o una clave en texto plano en `server.json`).

## Añadir servidores

`-a` guarda un servidor en `client.json` y sale — nunca abre un terminal. Dos formas:

- `vtermc -a` — avisos interactivos: `Server name`, `Server address` y `Secret key`. La dirección debe ser `wss://host:port` (la entrada no válida se vuelve a pedir; una dirección vacía cancela). Un nombre vacío recurre a la dirección;
- `vtermc -a "vtshare1:..."` — pega la cadena de uso compartido cifrada. Pulsar el `QR Code` de emparejamiento en la sección `Server` de la interfaz web la copia al portapapeles; `vtermc` la descifra y rellena nombre, dirección y clave secreta automáticamente.

Detalles que conviene saber:

- reañadir una dirección existente actualiza esa entrada en su lugar (conserva su id) en lugar de duplicarla;
- las direcciones locales se rechazan — `Local` está siempre implícitamente disponible y nunca necesita una entrada;
- las entradas también las muestra el Server Manager del cliente de escritorio.

## Seleccionar servidores y sesiones

`vtermc -s` abre dos selectores a pantalla completa (requiere un terminal interactivo):

1. **Lista de servidores** — primero `Local`, luego cada servidor gestionado como `Name  (address)`. Navega con las flechas o `j`/`k`, salta con un dígito, `Enter` selecciona, `q` cancela;
2. **Lista de terminales** — primero `New Terminal`, seguido de las sesiones activas de ese servidor con directorio de trabajo, título y servidor propietario. Adjúntate a una sesión existente, o crea una nueva (que luego respeta `-p` / `-w` como una ejecución simple de `vtermc`).

Adjuntarse nunca interrumpe la sesión — sigue ejecutándose en el servidor entre adjunciones.

### Eliminar un servidor

En la lista de servidores, muévete a un servidor gestionado y pulsa `d`: la fila se pone roja y el aviso pide confirmación. Pulsa `y` (o `Enter`) para eliminar la entrada de `client.json`, cualquier otra tecla aborta. `Local` no se puede eliminar.

## Variables de entorno

Una sesión nueva hereda un conjunto fijo de tu shell — `PATH`, `HOME`, `USER`, `SHELL`, `TERM`, `WT_PROFILE_ID`, `PSMODULEPATH`:

- `--env-inherit <patterns>` — lista separada por comas (o la variable de entorno `VTERM_ENV_INHERIT`); se admiten comodines `*`;
- `-e VAR=VALUE` — inyectar variables adicionales (repetible);
- `--no-env` — iniciar la sesión sin variables heredadas.

```sh
vtermc -w ~/project -e RUST_LOG=debug --env-inherit "PATH,HOME,TERM"
```

## Todas las opciones

```text
-p, --profile [<ID>]          Seleccionar perfil de terminal (omitir para listar interactivamente)
-w, --working-dir <DIR>       Establecer directorio de trabajo
-s, --select                  Seleccionar servidor y terminal existente interactivamente
-a, --add [<SHARE>]           Añadir un servidor (vtshare1: cadena de uso compartido, o interactivo)
    --server <URL>            URL WebSocket del servidor [env: VTERM_SERVER]
-e, --env <VAR=VALUE>         Variables de entorno adicionales
    --no-env                  No heredar variables de entorno
    --env-inherit <PATTERNS>  Patrones de variables de entorno a heredar [env: VTERM_ENV_INHERIT]
```
