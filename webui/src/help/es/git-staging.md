La zona `CHANGES` del panel Git cubre el bucle de commit cotidiano: revisar cambios, staging, descartar, stash y commit con mensaje.

## Stage y unstage

- los cambios sin staged se listan bajo `CHANGES`, cada archivo con una insignia de estado — `M` modificado, `A` añadido, `D` eliminado, `?` sin seguimiento, `!` conflicto (las filas de submódulo llevan una etiqueta `SUB` extra);
- pasar el puntero sobre una fila revela botones de acción en línea: `+` hace stage de un archivo sin staged, `−` unstage de uno staged, y `×` en una fila sin staged lo descarta; los botones `Stage All Changes` / `Unstage All Changes` de los encabezados de sección lo gestionan todo de una vez;
- `Ctrl`-clic alterna archivos a una selección múltiple y `Shift`-clic selecciona un rango — un botón de acción se aplica entonces a cada archivo seleccionado;
- pulsar un nombre de archivo abre su diff para revisar el cambio antes de decidir;
- el clic derecho sobre un archivo ofrece `View File History`, `Stash This File`, `Copy Full Path` / `Copy Relative Path`, `Reveal in File Explorer` y un atajo `cd` que lleva el terminal a la carpeta del archivo.

![La sección STAGED CHANGES — archivos staged con botones Unstage por fila](images/git-staging-staged.jpg)

## Hacer commit

1. Haz stage de lo que deba entrar;
2. escribe un mensaje en el cuadro de commit (`Ctrl+Enter` hace commit) o pulsa el botón IA para generar uno (véase Asistencia LLM);
3. pulsa `Commit`.

![El cuadro de commit — entrada de mensaje con el botón de generación IA](images/git-staging-commit.jpg)

Tras el commit, pasa el puntero sobre el encabezado del historial — la barra con el nombre de la rama sobre el historial de commits — y los botones `Fetch`, `Pull` y `Push` aparecen a su derecha. `Push` envía la rama actual a su upstream de inmediato; para opciones como `Force with lease`, usa el diálogo de push de la lista de ramas (véase Push, checkout y más).

![Pasar el puntero sobre el encabezado del historial revela los botones Fetch, Pull y Push](images/git-staging-push.jpg)

## Stash

- el botón de archivo del encabezado `CHANGES` abre el diálogo `Stash All Changes`, donde un mensaje opcional describe el stash;
- el clic derecho sobre archivos sin staged seleccionados ofrece `Stash This File` (o `Stash <n> Selected Files` para selección múltiple) para hacer stash solo de esas rutas — las filas de submódulo se excluyen;
- los stashes se reúnen bajo `STASHES` como entradas `WIP on <rama>`; desplegar uno lista los archivos capturados, y pulsar un archivo muestra su diff;
- pasar el puntero sobre una fila de stash muestra `Apply` (restaurar y conservar), `Pop` (aplicar y quitar) y `Drop` (quitar sin aplicar) — `Pop` y `Drop` piden confirmación primero, y el menú contextual añade `View Diff`;
- la sección `STASHES` permanece oculta mientras la lista de stashes esté vacía.

![El diálogo Stash All Changes con su mensaje opcional](images/git-staging-stash-all.jpg)

![La sección STASHES — un stash desplegado a sus archivos, con botones Apply, Pop y Drop](images/git-staging-stashes.jpg)

![Pop aplica el stash y lo quita de la lista tras una confirmación](images/git-staging-stash-pop.jpg)

## Descartar

`Discard Changes` sobre un archivo sin staged tira todos sus cambios sin confirmar — hay un diálogo de confirmación y la acción **no puede deshacerse**.

> Un commit solo incluye contenido staged; los cambios sin staged permanecen en el árbol de trabajo.
