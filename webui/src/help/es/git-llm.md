La inteligencia del panel Git es impulsada por LLM: mensajes de commit con un clic, lectura de diffs asistida por IA y resolución de conflictos por agente, hunk a hunk. Todo depende de un proveedor LLM configurado en la interfaz web.

## Configurar el proveedor

Abre la sección `LLM` de la interfaz web:

- elige un `Provider` y rellena sus credenciales (los campos los define el manifiesto del proveedor); la configuración es global — la clasificación de terminal, los mensajes de commit de Git, el asistente de chat y otras funciones de IA comparten el mismo proveedor;
- los proveedores son addons de tipo LLM: los comunes vienen integrados, y los proveedores personalizados pueden subirse como zip en la sección `Addons` (el manifiesto declara los campos de configuración que la interfaz web convierte en formulario).

![La sección LLM de la interfaz web — elige un proveedor y rellena sus credenciales](images/git-llm-provider-config.jpg)

## Funciones LLM en Git

- **Generación de mensajes de commit** — pulsa el botón IA junto al cuadro de commit; el LLM lee el diff staged y escribe un mensaje, tecleado en el cuadro con una animación. El mensaje sigue siendo editable — haz commit solo cuando estés conforme;
- **Análisis de diff** — el botón destello del encabezado de la vista de diff analiza el diff actual;
- **Agente IA de conflictos** — el modo IA del editor de fusión del rebase resuelve hunks uno a uno con explicaciones y confianza (véase Resolver conflictos).

![El mensaje de commit generado por IA tecleado en el cuadro de commit](images/git-llm-commit-message.jpg)

> El LLM solo lee contenido del repositorio para producir texto; los commits y fusiones siempre ocurren cuando tú los confirmas.
