Los comandos rápidos fijan los comandos build/test/deploy de un proyecto en el menú contextual del terminal — un clic para ejecutar, disponible en todas partes. Los comandos se mantienen por repositorio en `.vterm/commands.json` en la raíz del repositorio.

## Crear commands.json

Clic derecho dentro de un terminal → `Commands`:

- cuando el repositorio de la sesión aún no tiene commands.json, el menú ofrece una única entrada, `Generate commands.json` — al pulsarla el servidor detecta el tipo de proyecto (npm / Cargo / .NET / Xcode / Gradle-Android / genérico) y escribe una plantilla inicial conservadora. Un archivo existente no vacío **nunca se sobrescribe**;
- ubicación del archivo: `.vterm/commands.json` en la raíz del repositorio Git (el directorio de trabajo actual para carpetas no Git).

## Refinar

La plantilla generada solo contiene los comandos habituales del tipo de proyecto detectado. Deja que tu agente de código (por ejemplo un agente de editor conectado al MCP de este servidor) lea el proyecto y refine el archivo:

```json
{
  "commands": [
    { "name": "Build", "command": "cargo build --release" },
    { "name": "Test verbose", "command": "cargo test -- --nocapture",
      "env": { "RUST_LOG": "debug" } },
    { "name": "Package (Windows)", "command": "./packaging/windows/package.bat",
      "os": "win" }
  ]
}
```

También se acepta un array de nivel superior; `env` es opcional y define variables antes de ejecutar el comando; la clave heredada `value` se sigue leyendo. El campo opcional `os` limita una entrada al sistema operativo donde corre `vterm-server`: usa `"win"` para Windows, `"linux"` para Linux u `"osx"` para macOS. Si `os` se omite, se establece en `"all"` o tiene un valor no reconocido, la entrada está disponible en todas las plataformas. El servidor vigila el CWD y los cambios de commands.json y los difunde — vuelve a abrir el menú y los comandos nuevos están ahí.

## Ejecutar

Clic derecho → `Commands` → pulsa una entrada para ejecutarla en ese terminal (el env opcional se aplica primero). El menú no está disponible mientras una aplicación de agente se ejecuta en la sesión, para que el texto del comando no caiga dentro de la interfaz de la aplicación.

## Ejemplo: compilación e instalación Android por Wi-Fi

Un recorrido completo en un proyecto Gradle-Android: fijar la configuración de ADB inalámbrico y el flujo de compilación-instalación en el menú contextual, y luego recompilar la app con un clic.

**1. Pasar el teléfono a ADB inalámbrico.** Estos dos comandos se escriben una vez por dispositivo — `adb tcpip 5555` mientras el teléfono está en USB, luego `adb connect` con la dirección del teléfono:

![Terminal — pasar un teléfono Android a ADB inalámbrico con adb tcpip 5555 y adb connect](images/terminal-commands-adb-setup.gif)

**2. Dejar que el agente de código escriba los comandos.** Abre el agente de código del proyecto (aquí Claude Code) en un terminal y pídele añadir comandos de compilación e instalación — lee la estructura del proyecto y escribe las entradas en `.vterm/commands.json`:

![Terminal — Claude Code editando commands.json con los comandos de compilación e instalación del APK](images/terminal-commands-agent-edit.mp4)

**3. Compilar e instalar desde el menú.** Divide el terminal para que la salida de compilación tenga su propio panel, luego clic derecho → `Commands` → la entrada — Gradle ensambla el APK y lo instala en el teléfono por Wi-Fi:

![Terminal — dividir un panel y ejecutar el comando de compilación e instalación Android desde el menú contextual](images/terminal-commands-split-run.mp4)
