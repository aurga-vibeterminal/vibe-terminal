Los agentes de código (Claude Code, Codex, Kimi, Pi, OpenCode…) solo saben lo que llega a su entrada. `Añadir contexto` es la forma de entregarles rutas de archivos y fragmentos de código sin reescribir: arrastra archivos desde el `Explorer`, o arrastra texto seleccionado desde el editor de código y la vista de diff de Git, directamente al terminal del agente o a la entrada de chat del Copilot.

## Arrastrar archivos

Pulsa un archivo — o una selección múltiple — en la lista de archivos del `Explorer` y arrástralo:

- **Sobre un terminal**: las rutas completas se teclean en el punto de soltar, separadas por espacios, listas para formar parte de tu próximo mensaje al agente;
- **Sobre la entrada de chat del Copilot**: las rutas se insertan en el borrador del mensaje, una por línea, para que armes un mensaje multirchivo antes de enviar.

Las rutas arrastradas desde el gestor de archivos de tu sistema operativo y soltadas sobre un terminal se entrecomillan automáticamente, de modo que los nombres con espacios sobreviven intactos.

![Arrastrar un archivo de la lista del Explorer a un terminal teclea su ruta completa](images/terminal-context-drag-file.gif)

## Arrastrar código seleccionado como contexto

Selecciona texto en el editor de código o en un diff de Git, luego pulsa la selección misma y arrástrala a un terminal:

- cuando el terminal **ejecuta un agente de código**, el soltar inserta un *bloque de contexto* — la ruta del archivo, el rango de líneas y el fragmento envuelto en una valla de código:

````text
File: crates/vterm/src/ui/terminal_view.rs
Lines: 558-566
```
fn can_accept_native_drop(&self, info: &DndInfo) -> bool { … }
```
````

- cuando el terminal está en un **aviso de shell simple**, el soltar teclea en cambio el texto seleccionado bruto, para que el mismo gesto sirva también para comandos ordinarios;
- cuando el diff se ve en un commit concreto, el bloque lleva además una línea `Commit:` que fija la revisión exacta.

![Arrastrar una selección del editor a un terminal de agente suelta un bloque de contexto File/Lines](images/terminal-context-drag-selection.mp4)

## Copiar contexto

El mismo bloque vive en el menú contextual: `Copy Context` en el editor de código y la vista de diff lo copia al portapapeles. Pégalo en la entrada de chat del Copilot — o en cualquier otro agente — cuando arrastrar sea poco práctico. La animación de arriba muestra el bloque resultante en la entrada del agente.

## Consejos

- Selección múltiple en el `Explorer` para soltar varias rutas de una vez;
- la entrada de chat del Copilot envía directo al terminal como entrada tecleada (véase `Historial del agente de código`), así que el contexto añadido allí alcanza al agente exactamente como si lo hubieras tecleado;
- combina con `Confirmación automática` para que el agente actúe sobre el contexto soltado sin más avisos.
