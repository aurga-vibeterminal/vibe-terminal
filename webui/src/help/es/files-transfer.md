Las transferencias de archivos entre tu máquina y el servidor son arrastrar y soltar, y la dirección es el significado: **arrastrar hacia dentro = subir al servidor, arrastrar hacia fuera = descargar a esta máquina**. Las transferencias viajan por la conexión cifrada cliente-servidor; los servidores remotos y compartidos se comportan como los locales.

## Subir: arrastrar hacia dentro

Arrastra archivos (o carpetas enteras) desde el gestor de archivos de tu sistema operativo sobre la lista de archivos y suelta:

- en modo árbol, **soltar sobre una fila de carpeta** sube a esa carpeta; **soltar en la raíz** (la línea de inserción entre filas) sube al directorio raíz actual;
- aparece una fila de transferencia al pie de la lista mostrando `Uploading`, el nombre del archivo y una barra de progreso;
- al completarse muestra `Uploaded n file(s)`; los fallos incluyen el motivo (p. ej. permiso denegado) y la lista se refresca para revelar los archivos nuevos.

## Descargar: arrastrar hacia fuera

Pulsa un archivo de la lista, arrástralo fuera de la ventana del cliente y suéltalo en el gestor de archivos de tu sistema operativo (o el escritorio):

- el archivo fluye desde el servidor hasta donde lo soltaste;
- las filas de transferencia muestran el progreso igualmente; **una fila de descarga terminada es pulsable** y abre la carpeta local de descargas;
- selección múltiple y arrastra varios archivos de una vez.

![Progreso de transferencia — las filas de subida/descarga](images/files-transfer.jpg)

> Arrastrar y soltar mueve archivos individuales entre esta máquina y un servidor; para mover datos entre dos servidores, combina Publicar con servidor HTTP o comandos de terminal (scp/rsync).
