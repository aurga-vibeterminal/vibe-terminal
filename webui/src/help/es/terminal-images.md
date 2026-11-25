Las capturas de pantalla y el terminal se encuentran en un mismo flujo: pulsa un único atajo global para tomar cualquier región del escritorio, anótala, y cópiala o guárdala — luego pega la imagen del portapapeles directamente en un terminal, donde aterriza como ruta de archivo que el programa en ejecución (un agente de código incluido) puede abrir y usar. Ambas mitades viven en el cliente de escritorio.

## Hacer una captura

- Pulsa `Shift`+`Alt`+`A` en cualquier lugar: el atajo es global, así que se dispare sin importar qué aplicación tenga el foco mientras el cliente de escritorio esté en marcha (en macOS, pulsa `Shift`+`Command`+`A`);
- Cada pantalla se atenúa bajo una capa de captura. Arrastra para dibujar la selección — puede abarcar varias pantallas —, luego arrastra dentro de ella para moverla o tira de sus tiradores para redimensionarla; una placa sobre la selección muestra su tamaño en píxeles;
- Pulsa `Esc` en cualquier momento para cancelar.

![La capa de captura — una selección con su placa de tamaño y la barra de anotación](images/terminal-images-capture.jpg)

## Anotar

La barra bajo la selección lleva a la izquierda las herramientas de anotación (rectángulo y elipse), en el medio seis colores y a la derecha tres botones de acción — cancelar (`✕`), guardar (flecha de descarga) y copiar (`✓`).

- Elige una herramienta y un color, luego arrastra dentro de la selección para dibujar; los trazos se recortan a la selección;
- `Ctrl`+`Z` deshace la última anotación, `Ctrl`+`R` la rehace;
- Arrastrar una zona nueva fuera de la selección actual la descarta — anotaciones incluidas — y empieza otra.

## Copiar o guardar

- `Copiar` (`✓`) pone la captura anotada en el portapapeles — lista para pegarla en un terminal (abajo), un chat o cualquier otra aplicación;
- `Guardar` (flecha de descarga) escribe un PNG mediante un diálogo de guardado; el nombre de archivo por defecto es `vterm-screenshot-<timestamp>.png`.

## Pegar una imagen en un terminal

- Copia una imagen al portapapeles — desde la herramienta de captura de arriba, un navegador o la captura propia del sistema operativo;
- Haz clic derecho en el terminal y elige `Paste Image`; la entrada solo aparece cuando el portapapeles contiene realmente una imagen;
- La imagen se sube al servidor de la sesión y se guarda allí como archivo temporal; su ruta entre comillas se inserta después en el cursor del terminal exactamente como si la hubieras tecleado — envuelta en marcadores de pegado con corchetes cuando la aplicación en marcha los soporta, de modo que shells, editores y agentes la reciben limpiamente;
- Para el programa es texto plano: un agente de código como Claude Code u opencode abre la imagen desde la ruta, y los comandos ordinarios del shell pueden usarla como argumento.

![Clic derecho del terminal — la entrada Paste Image](images/terminal-images-paste.jpg)

## Notas

- La ruta insertada vive en el servidor de la sesión: en un servidor remoto o WSL el archivo temporal se crea en esa máquina, justo donde los programas que corren en el terminal pueden alcanzarlo;
- Combinado con `Añadir contexto a los agentes`, es la manera más rápida de mostrarle a un agente algo de tu pantalla — captura, pega, y el agente tiene la imagen como archivo;
- En Linux la captura funciona bajo X11; las sesiones Wayland nativas no soportan ni el atajo global ni la capa interactiva, y si otra aplicación X11 ya tomó `Shift`+`Alt`+`A`, el atajo no puede registrarse.
