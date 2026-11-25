Los worktrees extraen varias ramas de un repositorio a directorios separados: desarrolla en paralelo sin tocar el árbol de trabajo principal, luego fusiona la rama de vuelta y limpia. Vibe Terminal convierte todo el ciclo en apuntar y hacer clic.

## Crear un worktree

1. En la lista de ramas del panel Git, clic derecho en la rama destino (local o remota) → `New Worktree`;
2. el diálogo nombra el worktree nuevo por la rama — edítalo a tu gusto; confirma, y el servidor crea el directorio del worktree y extrae allí la rama;
3. **se abre automáticamente una pestaña de terminal nueva** con su directorio de trabajo en el worktree nuevo — empieza a desarrollar aislado de inmediato mientras el árbol principal permanece en su rama.

![El diálogo New Worktree — el nombre es por defecto la rama](images/git-worktree-create.jpg)

![La pestaña nueva abierta para el worktree recién creado](images/git-worktree-terminal.jpg)

## Trabajar en paralelo

Cada worktree es un directorio de trabajo totalmente independiente: las compilaciones y pruebas de la rama principal y de la rama de función corren lado a lado en pestañas distintas sin interferencias. Las ramas extraídas en un worktree llevan un marcador de worktree que muestra su ruta en la lista de ramas, y commit y push funcionan desde la pestaña del worktree como en cualquier rama.

El clic derecho sobre una rama de worktree añade dos entradas exclusivas: `Reveal in File Explorer` (sesiones locales) abre su directorio en el gestor de archivos del sistema, y `Open in Terminal` abre otra pestaña allí.

## Fusionar de vuelta y limpiar

1. Haz commit y push dentro de la pestaña del worktree como de costumbre;
2. cambia la pestaña principal a la rama destino (p. ej. `main`), clic derecho en la rama de función → `Merge into Current Branch` para traer el trabajo;

   ![Fusionar la rama del worktree de vuelta a la rama principal](images/git-worktree-merge.jpg)

3. una vez fusionado, clic derecho en la rama → `Delete Worktree`: esto elimina juntos el directorio del worktree y su rama. Asegúrate antes de que todo esté fusionado o empujado — el diálogo vuelve a advertir y la acción no puede deshacerse.

   ![La confirmación Delete Worktree — directorio y rama se eliminan juntos](images/git-worktree-delete.jpg)

