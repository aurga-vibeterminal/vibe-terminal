El rebase interactivo convierte `git rebase -i` en un flujo visual: arrastra para reordenar la cola de commits, aplica los commits uno a uno, resuelve conflictos visualmente — sin línea de comandos.

## Empezar

Abre el panel de ramas y escribe en el cuadro `Filter branches...` para encontrar la rama destino, luego clic derecho sobre ella → `Rebase Current Branch onto '{branch}'` para rebasar la rama actual sobre el destino. El panel cambia a la vista de rebase, que avanza por cuatro fases:

![Búsqueda de rama en el panel, luego inicio del rebase sobre el destino](images/git-rebase-search.jpg)

## 1. Editar la cola de commits

Los commits de tu rama que al destino le faltan forman una cola:

- **arrastra** para reordenarlos;
- cada commit tiene un desplegable de acción: `pick` (conservar) / `squash` (plegar en el anterior, fusionar mensajes) / `fixup` (plegar, descartar mensaje) / `edit` (detenerse para corregir) / `drop` (descartar);
- los commits squashed y droppados se pliegan visualmente.

![La cola de commits del rebase — reordenar arrastrando, acciones por commit](images/git-rebase-queue.jpg)

## 2. Aplicar

`Start Rebase` reproduce la cola sobre el destino commit a commit con una barra de progreso en vivo; sin conflictos corre directa hasta terminar. `Abort` retrocede en cualquier momento.

## 3. Conflictos (si los hay)

Un conflicto durante la aplicación cambia a la vista de conflictos — una lista de archivos más el editor de fusión con sus cuatro modos (`Merge` / `Split` / `Raw` / `AI`), donde cada hunk en conflicto recibe su propia decisión (`Ours` / `Theirs` / `Both`…; véase Resolver conflictos). Una vez resuelto cada archivo, `Continue` reanuda el rebase.

![La vista de conflictos — decisiones de fusión por hunk](images/git-rebase-resolve-conflict.jpg)

## 4. Hecho

Al completarse el panel confirma `Branch '{branch}' was rebased onto '{target}'` y muestra la tarjeta de push — el rebase reescribió el historial, así que la rama remota debe actualizarse a la fuerza:

- `Force Push with Lease` — recomendado: solo sobrescribe cuando nadie más ha empujado y nunca usa un `--force` simple;
- `Not now` — saltar el push; el rebase local ya está completo.

![La página de fin — Force Push with Lease o saltar el push](images/git-rebase-done.jpg)

> La sesión sigue utilizable durante el rebase; cerrar el panel no interrumpe uno en curso.
