Las variables de entorno te permiten mantener conjuntos agrupados de variables por proyecto (`ANDROID_HOME`, claves de API, ajustes de proxy) y aplicarlas o retirarlas en cualquier sesión de terminal con un clic. Viven en la configuración del servidor; los valores se cifran en reposo.

## Mantenimiento en la interfaz web

La sección `Environment Variables` las organiza como un árbol:

- **Grupos** — contenedores puros, anidables (clic derecho → `New Group`), para organizar por proyecto o escenario;
- **Colecciones** — las hojas del árbol y las auténticas portadoras de variables (`New Leaf`); solo las colecciones pueden aplicarse a un terminal;
- Los pares clave/valor de cada colección se editan con `Edit Variables` (`Add Variable`); los valores están ocultos por defecto;
- Las colecciones pueden duplicarse para derivar variantes (el mismo conjunto con una clave cambiada).

![La página Environment Variables — el árbol de grupos/colecciones](images/terminal-env-tree.jpg)

## Aplicar a un terminal

Clic derecho dentro de un terminal → `Envs`:

- el submenú refleja el árbol de la interfaz web; despliega una colección y elige `Apply` para inyectar sus variables en la sesión actual, o `Clear` para retirarlas;
- los procesos recién iniciados ven las variables (los programas en primer plano ya en ejecución no);
- las colecciones aparecen plegadas bajo sus grupos; el elemento `Envs` entero se oculta cuando el árbol no tiene colecciones.

![Clic derecho del terminal — aplicar y retirar en el submenú Envs](images/terminal-env-menu.mp4)
