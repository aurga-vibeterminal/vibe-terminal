El explorador de archivos opera sobre el sistema de archivos del lado servidor. La barra de herramientas en la parte superior del panel izquierdo alterna entre Recent / Explorer / Git / Debug / Search; Explorer es el navegador de archivos.

## El botón Home: saltar al cwd del terminal

Pulsa el botón `Home` (casa) de la barra de herramientas:

- Cambia a la vista `Explorer`;
- Salta al **directorio de trabajo del terminal activo** (su cwd);
- Fija ese directorio como árbol y limpia el cuadro de búsqueda — siempre partes de «donde esté el terminal, eso es lo que ves».

![El explorador de archivos en modo normal — Home, las migas de pan y la lista plana](images/files-explorer.jpg)

## Modo árbol y búsqueda

- **Fijar como árbol**: el botón de chincheta del extremo derecho de las migas de pan alterna entre la lista plana y el modo árbol. En modo árbol las carpetas se expanden en un árbol de directorios plegable (el salto de Home fija automáticamente);
- **Búsqueda**: el cuadro de búsqueda superior filtra la lista actual por nombre mientras escribes; `Search in Folder` del menú contextual de Recent ejecuta una búsqueda recursiva sobre un directorio entero.

![El explorador de archivos en modo árbol — la raíz fijada expandiéndose en un árbol de directorios plegable](images/files-explorer-tree.jpg)

## Operaciones básicas

Clic derecho en un archivo o carpeta:

- **Crear**: `New Folder` / `New File`;
- **Editar**: `Rename`; `Copy` / `Cut` / `Paste` (funciona entre directorios);
- **Eliminar**: `Delete`, con diálogo de confirmación;
- **Enlace con el terminal**: `Open in Terminal` abre una pestaña de terminal nueva en ese directorio; `cd to Directory` hace cd del terminal actual hacia allí;
- **Enlace con el sistema**: `Reveal in File Explorer` muestra un directorio local en el gestor de archivos del sistema;
- **Rutas**: `Copy Full Path` / `Copy Relative Path`;
- **Publicar**: `Host with HTTP server` publica el directorio como servicio de archivos HTTP (véase Publicar con servidor HTTP); `Open in App` lo abre con una aplicación de agente detectada en el servidor.

Cada operación actúa sobre el sistema de archivos del lado servidor; las sesiones remotas se comportan exactamente como las locales.
