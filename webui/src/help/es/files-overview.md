La gestión de archivos te permite trabajar con archivos del servidor directamente desde el cliente: la barra de actividad izquierda tiene dos entradas — `Recent` (proyectos recientes) y el explorador de archivos — usadas junto a las pestañas de terminal. Todo opera en el servidor de la sesión: la máquina local, o una máquina remota/compartida a través de un relay.

## Recent: proyectos recientes

`Recent` lista las carpetas que tus agentes de código abrieron más recientemente en el servidor actual, lo más nuevo primero:

- **Pulsa** una entrada y el explorador de archivos cambia a ese directorio;
- **Clic derecho** abre un menú de acciones rápidas:
  - `Open in VS Code` — abrir el directorio en VS Code en esta máquina;
  - `Reveal in File Explorer` — mostrarlo en el gestor de archivos del sistema;
  - `Copy Full Path` / `Copy Relative Path` — ruta completa, o ruta relativa al repositorio;
  - `Search in Folder` — buscar archivos dentro de ese directorio;
  - `Open in Terminal` — abrir una pestaña de terminal nueva allí; `cd Here` — hacer cd del terminal actual hacia él;
  - `Host with HTTP server` — publicar el directorio como servicio de archivos HTTP (véase Publicar con servidor HTTP);
  - `Open in App` — abrirlo con una aplicación de agente detectada en el servidor.

![El panel izquierdo — la lista de proyectos recientes](images/files-overview.jpg)

## El explorador de archivos

La pestaña `Explorer` navega y gestiona el sistema de archivos del lado servidor: navegación de directorios, vistas árbol y plana, búsqueda, crear/renombrar/eliminar y otras operaciones de archivos, más subida y descarga por arrastrar y soltar entre el servidor y esta máquina. Véase Explorador de archivos y Subir y descargar.
