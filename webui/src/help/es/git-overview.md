El panel Git incorpora el control de versiones en el cliente: staging, commits, gestión de ramas, worktrees, rebase interactivo, resolución de conflictos y push — sin salir de tu ventana de terminal. Todo opera en el repositorio Git de la sesión de terminal actual: en el directorio que se ejecute la sesión está el repositorio que muestra el panel.

## Abrirlo

Pulsa la pestaña `Git` en la barra de actividad izquierda (los datos del repositorio solo aparecen cuando la sesión activa está dentro de un repositorio Git). El repositorio de una pestaña sigue su sesión activa; las migas de pan del repositorio permiten saltar entre repositorios padre e hijo, y los submódulos se listan aparte.

![El panel Git — cambios, historial y ramas](images/git-overview-panel.jpg)

## Qué puede hacer

- **Staging y commits** — stage/unstage por archivo o en bloque, descartar cambios, commit con mensaje, mensajes de commit generados por IA (véase Asistencia LLM);
- **Historial** — historial de commits y por archivo, comparar y restaurar entre revisiones;
- **Ramas** — checkout, crear y eliminar ramas locales/remotas y etiquetas;
- **Worktrees** — extraer una rama a un árbol de trabajo aislado (se abre automáticamente una pestaña de terminal nueva), fusionarla de vuelta al terminar (véase Worktrees);
- **Rebase interactivo** — reordenar/squash/editar visualmente la cola de commits con resolución de conflictos (véase Rebase interactivo);
- **Resolución de conflictos** — un editor de fusión con cuatro modos, incluido un agente de IA que resuelve hunks uno a uno (véase Resolver conflictos);
- **Push y varios** — un diálogo de push completo, stashes, manejo de HEAD desacoplado (véase Push, checkout y más).

Las páginas siguientes cubren cada área en ese orden.
