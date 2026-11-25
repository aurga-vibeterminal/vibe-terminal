Conectar una máquina al relay requiere dos campos: la dirección del relay y su clave. Véase la página anterior, Descripción general, para qué es la función y cómo encajan las piezas.

## Configurar el servidor relay

1. Abre la sección `Server` en la máquina que actuará como relay.
2. Asegúrate de que haya una `Secret Key` establecida — el botón de dado genera una. La misma clave empareja clientes y servidores compartidos.
3. Despliega el bloque `QR Code` y elige la `Network Interface` / `IP Version` que clientes y servidores compartidos usarán para alcanzarlo.

## Conectar un servidor compartido

Los datos de conexión del relay viajan en el portapapeles — nada de reescribir:

1. En la interfaz web del servidor relay, pulsa el `QR code`. Esto copia una cadena de uso compartido cifrada (dirección, puerto, clave secreta) al portapapeles.
2. Abre la interfaz web en la máquina a compartir, sección `Server`, y haz clic en el campo `Relay Address`.
3. El navegador muestra un diálogo preguntando si permitir a este sitio acceder al portapapeles — pulsa `Allow`:

![Navegador — el diálogo de permiso de acceso al portapapeles mostrado al enfocar el campo](images/relay-allow-paste.jpg)

4. Con el acceso concedido, se comprueba el portapapeles: cuando contiene la cadena de uso compartido, aparece un botón de pegar junto al campo — púlsalo para descifrar y autorrellenar `Relay Address` y `Relay Secret Key`:

![Interfaz web — dirección de relay y clave secreta autorrellenadas desde la cadena pegada](images/relay-paste-share.jpg)

5. Revisa los valores y pulsa `Apply` — guardar reconecta el relay aguas arriba.

![Interfaz web — los campos de relay rellenados con el botón Apply](images/relay-apply-share.jpg)

> Leer el portapapeles exige un contexto seguro (HTTPS o localhost). En HTTP simple el botón de pegar permanece oculto; escribe a mano la dirección y la clave del relay.

Por defecto, la conexión aguas arriba está cifrada pero no verificada. Para fijar también el certificado del relay, ver Verificación estricta en el relay en Certificados y TLS.

## Cliente de escritorio

### Pestañas de servidores compartidos

Las pestañas de terminales que corren en un servidor compartido llevan dos marcadores: una franja de color en el borde izquierdo de la pestaña — un color por servidor — y el icono del SO remoto (Windows / Apple / Linux) junto al punto de estado. Las pestañas locales del propio servidor relay no tienen marcadores.

![Cliente de escritorio — marcadores de pestaña de terminales de servidores compartidos](images/relay-tab-indicator.jpg)

### Perfiles de terminal por servidor

El menú de perfiles (barra de título) agrupa los perfiles por servidor: primero los del propio servidor relay conectado, luego un grupo plegable por servidor compartido, nombrado por la máquina. Despliega un grupo y elige un perfil para abrir un terminal en ese servidor. El menú sigue al grupo de pestañas seleccionado en la barra de título.

![Cliente de escritorio — menú de perfiles con grupos por servidor](images/relay-profile-menu.jpg)

### Agrupación y ventanas separadas

El selector de grupos de la barra de título lista los grupos de pestañas de todos los servidores; el grupo por defecto de un servidor compartido se muestra como `Ungrouped (Nombre del servidor)`. El submenú de cada grupo lista sus pestañas para cambiar rápido y ofrece `Move Group to New Window`, que arranca el grupo entero a su propia ventana — útil para dedicar una ventana o monitor a una máquina.

![Cliente de escritorio — menú del selector de grupos con la acción mover a nueva ventana](images/relay-group-selector.jpg)
