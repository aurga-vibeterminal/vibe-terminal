La traducción convierte el texto que encuentras en el cliente al idioma que elijas: respuestas del chat de IA, mensajes de commit, comentarios de código o cualquier selección en el editor de código o en un diff. El trabajo lo hace un modelo de traducción configurado en el servidor — el cliente solo envía el texto, renderiza el resultado y lo guarda en caché para que puedas alternar entre el original y la traducción.

## Dónde está disponible la traducción

- **Panel de Copilot** — cada encabezado de mensaje (los tuyos, las respuestas del modelo y los bloques `Pensando...`) lleva un icono de traducción. Un clic reemplaza el cuerpo del mensaje por su traducción; otro clic restaura el original;
- **Borrador del mensaje de commit** — la caja de commit del panel Git muestra un botón de traducción en cuanto tu borrador ocupa varias líneas. Traduce el borrador completo en el sitio, y otro clic devuelve tu redacción original;
- **Popup de detalles del commit** — pasar el cursor sobre un commit en la historia de Git abre un popup de detalles; el icono de traducción junto al hash cambia el mensaje de commit a tu idioma;
- **Editor de código y vista de diff** — selecciona lo que quieras (las selecciones de bloque también valen), clic derecho y elige `Traducir`. Una insignia de globo marca las líneas traducidas en el margen, y una tarjeta con la traducción se abre junto a la selección.

## Cómo funciona

1. El cliente envía el texto al servidor — junto con el nombre del archivo cuando el texto se seleccionó en el editor o en un diff, para que el modelo sepa qué sintaxis de comentarios usa el archivo;
2. El LLM de traducción del servidor lo traduce al idioma de destino configurado en la interfaz web. El formato Markdown (encabezados, listas, tablas, bloques de código) se conserva; en selecciones de archivos, los marcadores de comentario como `//` o `#` se reconocen como decoración y se eliminan, mientras que el código y los identificadores quedan intactos;
3. El cliente renderiza el resultado como Markdown allí de donde vino la petición — en el sitio para mensajes de chat, o en una tarjeta desplazable para selecciones del editor y de diffs.

## Qué necesitas

- Un servidor con la función de traducción habilitada — la configuración única está en *Configuración del servidor* (sección `LLM` de la interfaz web);
- Un cliente de escritorio conectado a ese servidor. No se configura nada del lado del cliente: todas las superficies usan automáticamente los mismos ajustes del servidor.

Si la traducción no está configurada, el cliente muestra el error del servidor (por ejemplo en la tarjeta de traducción) en lugar de un resultado.
