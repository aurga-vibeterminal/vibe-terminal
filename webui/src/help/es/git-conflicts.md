Cuando un rebase o fusión choca con conflictos, el panel cambia a la vista de conflictos: la lista de archivos en conflicto a la izquierda, el editor de fusión a la derecha. Los archivos en conflicto también aparecen en la lista `CHANGES` con una insignia `!` — pulsar uno salta directamente a esta vista en lugar de un diff simple. El editor ofrece cuatro modos de resolución y puedes cambiar entre ellos en cualquier momento.

![La vista de conflictos — lista de archivos a la izquierda, editor de fusión a la derecha](images/git-conflicts-view.jpg)

## La lista de archivos en conflicto

Los archivos se agrupan por estado: los autofusionados con éxito (staged automáticamente) y los que aún requieren atención manual. Pulsa un archivo para abrirlo en el editor; la lista se actualiza en vivo mientras resuelves, hasta quedar vacía.

## Los cuatro modos de resolución

- **Merge** (por defecto) — los conflictos se presentan hunk a hunk; por hunk elige `take ours` / `take theirs` o edita el resultado fusionado en línea;
- **Split** — un diff se rompe en segmentos más finos a nivel de línea, para adoptar ambos lados trozo a trozo dentro de un hunk;
- **Raw** — edición simple con marcadores de conflicto `diff3` (`<<<<<<<` / `=======` / `>>>>>>>`) para ajuste a mano;
- **AI** — un agente resuelve el archivo hunk a hunk: el LLM analiza cada hunk en conflicto y produce tarjetas de solución con explicación y confianza. Puedes teclear instrucciones para guiar la estrategia (p. ej. «prefiere theirs, conserva los logs»), pausar, afinar en otro modo y reanudar — los hunks ya resueltos se saltan.

En cada modo la resolución se escribe en el archivo de tu sesión y se hace stage, exactamente como una edición manual.

## Continuar el rebase

Una vez resuelto cada conflicto, el rebase pasa al commit siguiente; también puedes abortar en cualquier punto para volver al estado previo al rebase.
