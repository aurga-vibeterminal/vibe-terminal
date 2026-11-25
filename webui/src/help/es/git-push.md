El menú contextual de la lista de ramas concentra el push, el checkout y las operaciones cotidianas de ramas.

## Push

Clic derecho en una rama → `Push '{branch}'...` abre el diálogo de push:

![Clic derecho en rama — la entrada Push](images/git-push-contextmenu.jpg)

- muestra el mapeo de objetivos `local → remote`; cualquiera de los dos nombres puede cambiarse (empujar a una rama remota existente con otro nombre pide confirmación);
- opciones: `Push all branches`, `Set upstream` (vincular upstream en el primer push), `Push tags`, `Dry run`;
- el `Force` reescritor de historial y el más seguro `Force with lease` (rehúsa cuando otra persona ha empujado — preferido). Las operaciones que reescriben historial, como empujar tras un rebase, requieren uno de estos;
- el diálogo muestra un aviso de riesgo sobre los push forzados.

![El diálogo de push — objetivos local → remote y las casillas de opciones](images/git-push-dialog.jpg)

## Checkout

Clic derecho en una rama → `Checkout` para cambiar de rama — las ramas locales se extraen directamente, una rama remota o etiqueta abre el diálogo de checkout:

![Clic derecho en rama — la entrada Checkout](images/git-checkout-contextmenu.jpg)

- `New branch name` — la rama local a crear, prerrellenada desde la referencia;
- `Create new local branch` (`-B`), `Track remote as upstream` (`--track`, solo ramas remotas), `Force` (`-f`), `Merge` (`--merge`).

![El diálogo de checkout — nombre de rama y opciones](images/git-checkout-dialog.jpg)

El directorio de trabajo de la sesión sigue el cambio; el terminal y el panel Git reflejan la rama nueva de inmediato.

## HEAD desacoplado

Extraer un commit desnudo (p. ej. extraer un commit mientras navegas el historial) desacopla el HEAD. El siguiente commit abre entonces el diálogo `Detached HEAD`: elige una rama existente donde hacer commit, `+ Create new branch…` en su lugar, o `Skip` por ahora.

## Más operaciones

- **Stash** — haz stash de todos los cambios a la pila (mensaje opcional) desde el encabezado de cambios; la lista `STASHES` admite `Pop` (aplicar y quitar) y `Drop` (descartar, irreversible);
- **Eliminar ramas/worktrees** — vía el menú contextual de la entrada, ambas con confirmaciones de acción irreversible;
- **Historial y restauración** — `HISTORY` lista commits; clic derecho para historial de archivo, diff contra el árbol de trabajo o restaurar a una revisión;
- **Copiar** — nombres de ramas y hashes de commit se copian con un clic, práctico para pegar a un agente o un colega.
