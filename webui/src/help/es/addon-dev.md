Hay dos niveles de personalización: parchear la fuente de un addon existente, o escribir uno nuevo modeledo en un integrado y subirlo.

## Parcheo en caliente de la fuente

El botón de lápiz en cualquier tarjeta de addon abre su fuente JavaScript para editarla. Guardar almacena una instantánea que sobrescribe la original — tanto para addons integrados como subidos — y el cambio surte efecto de inmediato, sin reiniciar. Úsalo para ajustar un prompt, corregir una URL o prototipar una idea en el sistema en vivo.

`Reset` descarta la instantánea y restaura la fuente prístina (solo integrados — un addon subido ya es tuyo).

![Editar la fuente de un addon — la instantánea guardada sobrescribe la original](images/addon-edit-source.jpg)

## Escribir tu propio addon

Un addon son dos archivos:

- `<id>.json` — el manifiesto: `id`, `name`, `description`, `author`, `email`, `date`, `type` y `config_fields` (key, label, tipo de campo, valor por defecto, descripción) que la Web UI convierte en un formulario;
- `<id>.js` — la fuente: un archivo JavaScript normal (ejecutado por el motor QuickJS embebido, con un puente `addon.get_config()` hacia los campos configurados) que exporta las funciones que su tipo llama — p. ej. `chat_completion()` para un proveedor LLM, listados de herramientas y llamadas al puente para un addon MCP.

El arranque más rápido es exportar un addon existente: el botón `Export` en su tarjeta descarga un zip con el manifiesto y la fuente juntos. Elige el integrado más cercano a tu objetivo (también viven en `crates/vterm-server/src/addon/builtins/`), ajusta el id, name, type y las funciones exportadas, e instala el resultado con `Upload Addon`. Los addons subidos pueden borrarse de nuevo con el botón de papelera.

![Exportar un addon — manifiesto y fuente JavaScript en un solo zip](images/addon-export.jpg)

## Almacenamiento

Los addons subidos y las instantáneas de fuente viven en la carpeta `addons/` del directorio de configuración como `<id>.json` + `<id>.js`, así sobreviven a los reinicios. Para un id integrado solo se respeta la sobrescritura `.js` — el integrado conserva su propio manifiesto.
