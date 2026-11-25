Los terminales son el núcleo de Vibe Terminal: cada pestaña aloja una o más sesiones de terminal, puede dividirse en paneles o arrastrarse a su propia ventana. Los terminales alcanzan el servidor local así como servidores remotos y compartidos a través de un relay.

## Crear terminales

- Pulsa el botón `+` de la barra de título para abrir una pestaña con el perfil predeterminado;
- Pulsa el galón junto a `+` para el menú de perfiles: perfiles de terminal agrupados por servidor (local, conectados, compartidos) — despliega un grupo y elige un perfil para abrir un terminal en ese servidor;
- Arrastra una carpeta local a la barra de título y suéltala para abrir una pestaña nueva con ese directorio de trabajo (solo servidor local; el elemento soltado debe ser una carpeta).

![Barra de título — el botón de nueva pestaña y el menú de perfiles](images/terminal-usage-creation.jpg)

## Captura del ratón y menú contextual

- Clic derecho dentro de un terminal abre su menú contextual: pegar, dividir paneles (`Split`), comandos rápidos, colecciones de variables de entorno, copiar todo el texto, líneas de configuración MCP y más, según la sesión;
- Las aplicaciones que capturan el ratón (opencode, pi, vim, tmux, htop, …) reciben los eventos del ratón en su lugar — mantén `Shift` para realizar igualmente las operaciones habituales mientras corre esa app:
  - **Copiar texto**: mantén `Shift` y selecciona el texto, luego haz clic en otro sitio — `Shift` aún pulsado — para deseleccionar; la selección se copia al portapapeles;
  - **Menú contextual**: `Shift` + clic derecho o `Ctrl` + clic derecho abre el menú contextual; un clic derecho mientras se mantiene pulsado el botón izquierdo también lo abre (cualquier selección se descarta antes);
  - **Enlaces**: `Shift` + clic abre los enlaces con normalidad.

## Divisiones y organización por arrastre

- **Dividir**: clic derecho dentro de un terminal → `Split` → `Left` / `Right` / `Top` / `Bottom` divide la pestaña en paneles, cada uno un terminal independiente;

  ![Dividir una pestaña — el menú Split del clic derecho y los paneles resultantes](images/terminal-usage-split.gif)

- **Fusionar arrastrando**: arrastra una pestaña sobre otro terminal y aparecen zonas de acople (izquierda/derecha/arriba/abajo); suelta para fusionar la pestaña en la pestaña destino como nuevo panel;

  ![Fusión por arrastre — una pestaña soltada sobre otro terminal se convierte en nuevo panel](images/terminal-usage-merge.mp4)

- **Desprender a una ventana**: arrastra una pestaña fuera de la franja de pestañas (o sobre otra ventana) y toda la pestaña deja su ventana como una nueva. Las sesiones siempre se ejecutan en el servidor — divisiones, fusiones y arrastres nunca interrumpen un comando en ejecución.

## Pestañas y grupos de pestañas

- **Pestaña** — una pestaña de terminal, posiblemente con varios paneles divididos;
- **Grupo de pestañas** — un contenedor que organiza pestañas. Por defecto los grupos siguen a los servidores: las pestañas del servidor local viven en `Ungrouped`, y cada servidor compartido recibe un grupo `Name (Server)`; se admiten grupos personalizados;
- **Grupos y ventanas** — cada ventana muestra un grupo activo mediante el selector de grupos de la barra de título (o `All`); `Move Group to New Window` arranca un grupo entero a su propia ventana, dedicando una ventana o monitor a una máquina;
- **Grupos y servidores** — un grupo recuerda su servidor, así que cambiar de grupo reenfoca el espacio de trabajo de un servidor; el menú de perfiles sigue al grupo activo y solo muestra perfiles de servidores relevantes.

![El selector de grupos — lista de grupos y mover a nueva ventana](images/terminal-usage-groups.jpg)
