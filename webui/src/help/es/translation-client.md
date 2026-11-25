Una vez configurado el servidor, todas las superficies del cliente ofrecen traducción a través del mismo icono de globo. Los resultados se guardan en caché por elemento, así que alternar no repite peticiones.

## Mensajes del panel de chat

Cada encabezado de mensaje lleva un icono de traducción — junto a la hora en tus mensajes, a la derecha del nombre del remitente y la hora en las respuestas del modelo, y a la derecha de la etiqueta `Pensando...` en los bloques de razonamiento:

- **El primer clic** envía el Markdown en bruto del mensaje al servidor e intercambia el cuerpo por su traducción. El icono se ilumina mientras se muestra la traducción;
- **Otro clic** restaura el texto original;
- La traducción permanece en el mensaje mientras la conversación siga abierta, incluidos los bloques `Pensando...` una vez que los expandes.

Los mensajes traducidos se renderizan como Markdown normal, así que listas, tablas y bloques de código conservan su forma.

![Un mensaje de chat con el icono de traducción en el encabezado y el cuerpo traducido debajo](images/translation-chat-message.jpg)

## Borrador del mensaje de commit

En la caja de commit del panel Git, un botón de traducción aparece junto al botón de IA en cuanto tu borrador supera una línea:

- **Clic** para traducir el borrador completo — el texto traducido reemplaza el borrador en la caja como vista previa de solo lectura;
- **Clic de nuevo** para volver a tu redacción original (el borrador nunca se modificó);
- Mientras se muestra la vista previa no se puede escribir; vuelve primero y sigue editando.

![La caja de commit con el botón de traducción y la vista previa traducida](images/translation-commit-draft.jpg)

## Popup de detalles del commit

Pasa el cursor sobre un commit en la historia de Git para abrir el popup de detalles. Junto al botón de copiar el hash hay un icono de traducción: haz clic para traducir el mensaje de commit dentro del popup; haz clic de nuevo para ver el original. El mensaje traducido se renderiza como el original — línea de asunto, cuerpo Markdown y bloques de código incluidos.

![El popup de detalles del commit — botones de copiar y traducir en la línea del hash](images/translation-git-tip.jpg)

## Selecciones en el editor de código

Selecciona texto en el editor de código — las selecciones normales y de bloque (rectangulares) funcionan —, clic derecho y elige `Traducir` (la primera entrada del menú):

1. Una insignia de globo sólida aparece sobre el número de línea de la primera línea seleccionada;
2. Una tarjeta de traducción se abre de inmediato con un marcador *Translating…* y se rellena cuando el servidor responde;
3. La tarjeta es fija: se queda en su sitio mientras te desplazas, y solo su botón de cerrar, `Esc` o un clic fuera la descartan. Pasar el cursor sobre la insignia la vuelve a mostrar.

Cada selección traducida tiene su propia insignia, y traducir las mismas líneas otra vez simplemente reenseña la tarjeta en caché. Las insignias desaparecen al abrir otro archivo.

![Una selección en el editor de código con Traducir en el menú contextual](images/translation-editor-menu.jpg)

## Selecciones en la vista de diff

La vista de diff funciona igual: selecciona líneas del cuerpo del diff, clic derecho, `Traducir`. La insignia reemplaza el número de línea nueva de la primera fila seleccionada, y la tarjeta del resultado se ancla debajo. Las selecciones en modo edición también se traducen; cambiar a otro archivo o commit borra las insignias.

![Un diff con una selección traducida — insignia del margen y tarjeta de traducción](images/translation-diff-card.jpg)

## Qué se traduce

El servidor le dice al modelo de dónde viene el texto:

- **Prosa** (mensajes de chat, mensajes de commit) — el formato Markdown se conserva y solo se traduce el lenguaje natural;
- **Selecciones de archivo** (editor, diff) — el nombre del archivo viaja con la petición, así que el modelo conoce los marcadores de comentario del archivo (`//`, `#`, `--`, …). Los marcadores se eliminan y el texto del comentario se traduce como prosa; el Markdown dentro del comentario (encabezados, listas, bloques de código) sobrevive. Si la selección es código normal, solo se traducen los comentarios y los literales de cadena — los identificadores y la estructura quedan intactos.
